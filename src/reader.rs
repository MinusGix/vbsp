use crate::*;
use binrw::BinReaderExt;
use std::borrow::Cow;
use std::fmt::Debug;
use std::mem::size_of;

#[derive(Debug, Clone, Copy)]
pub struct Version(pub u32);

pub struct LumpReader<R> {
    inner: R,
    length: usize,
    lump: LumpType,
    version: Version,
}

impl<'a> LumpReader<Cursor<Cow<'a, [u8]>>> {
    pub fn new(data: Cow<'a, [u8]>, version: u32, lump: LumpType) -> Self {
        let length = data.len();
        let reader = Cursor::new(data);
        LumpReader {
            inner: reader,
            length,
            lump,
            version: Version(version),
        }
    }

    pub fn into_data(self) -> Cow<'a, [u8]> {
        self.inner.into_inner()
    }
}

impl<R: BinReaderExt + Read> LumpReader<R> {
    pub fn read_entities(&mut self) -> BspResult<Entities> {
        let mut entities = String::with_capacity(self.length);
        self.inner.read_to_string(&mut entities)?;
        Ok(Entities { entities })
    }

    /// Read a list of items with a fixed size
    pub fn read_vec<F, T>(&mut self, mut f: F) -> BspResult<Vec<T>>
    where
        F: FnMut(&mut LumpReader<R>) -> BspResult<T>,
    {
        if self.length % size_of::<T>() != 0 {
            return Err(BspError::InvalidLumpSize {
                lump: self.lump,
                element_size: size_of::<T>(),
                lump_size: self.length,
            });
        }
        let num_entries = self.length / size_of::<T>();
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(f(self)?);
        }

        Ok(entries)
    }

    /// Read a list of items with a fixed size and with the version
    pub fn read_vec_ver<F, T>(&mut self, mut f: F) -> BspResult<Vec<T>>
    where
        F: FnMut(&mut LumpReader<R>, Version) -> BspResult<T>,
    {
        if self.length % size_of::<T>() != 0 {
            return Err(BspError::InvalidLumpSize {
                lump: self.lump,
                element_size: size_of::<T>(),
                lump_size: self.length,
            });
        }
        let num_entries = self.length / size_of::<T>();
        let mut entries = Vec::with_capacity(num_entries);

        for _ in 0..num_entries {
            entries.push(f(self, self.version)?);
        }

        Ok(entries)
    }

    pub fn read<T: BinRead + Debug>(&mut self) -> BspResult<T>
    where
        T::Args<'static>: Default,
    {
        // let start = self.inner.stream_position().unwrap() as usize;
        let result = self.inner.read_le()?;
        // let end = self.inner.stream_position().unwrap() as usize;
        // todo: figure out how to only run this check for types that don't allocate
        // debug_assert_eq!(
        //     end - start,
        //     size_of::<T>(),
        //     "Incorrect number of bytes consumed while reading a {} ({:#?})",
        //     type_name::<T>(),
        //     result
        // );
        Ok(result)
    }

    pub fn read_args<T: BinRead + Debug>(&mut self, args: T::Args<'_>) -> BspResult<T> {
        let result = self.inner.read_le_args(args);

        Ok(result?)
    }

    pub fn read_visdata(&mut self) -> BspResult<VisData> {
        if self.length < size_of::<u32>() * 2 {
            return Ok(VisData::default());
        }

        let cluster_count = self.inner.read_le()?;
        let mut pvs_offsets = Vec::with_capacity(min(cluster_count as usize, 1024));
        let mut pas_offsets = Vec::with_capacity(min(cluster_count as usize, 1024));

        for _ in 0..cluster_count {
            pvs_offsets.push(self.inner.read_le()?);
            pas_offsets.push(self.inner.read_le()?);
        }

        let mut data = Vec::new();
        self.inner.read_to_end(&mut data)?;

        Ok(VisData {
            cluster_count,
            pvs_offsets,
            pas_offsets,
            data,
        })
    }
}

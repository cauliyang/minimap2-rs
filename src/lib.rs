use std::error::Error;
use std::mem::MaybeUninit;
use std::path::Path;

use minimap2_sys::*;

pub static MAP_ONT: &str = "map-ont\0";
pub static AVA_ONT: &str = "ava-ont\0";
pub static MAP10K: &str = "map10k\0";
pub static AVA_PB: &str = "ava-pb\0";
pub static MAP_HIFI: &str = "map-hifi\0";
pub static ASM: &str = "asm\0";
pub static SHORT: &str = "short\0";
pub static SR: &str = "sr\0";
pub static SPLICE: &str = "splice\0";
pub static CDNA: &str = "cdna\0";

#[derive(Debug)]
pub enum Preset {
    MapOnt,
    AvaOnt,
    Map10k,
    AvaPb,
    MapHifi,
    Asm,
    Short,
    Sr,
    Splice,
    Cdna,
}

impl From<Preset> for *const i8 {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::MapOnt => MAP_ONT.as_bytes().as_ptr() as *const i8,
            Preset::AvaOnt => AVA_ONT.as_bytes().as_ptr() as *const i8,
            Preset::Map10k => MAP10K.as_bytes().as_ptr() as *const i8,
            Preset::AvaPb => AVA_PB.as_bytes().as_ptr() as *const i8,
            Preset::MapHifi => MAP_HIFI.as_bytes().as_ptr() as *const i8,
            Preset::Asm => ASM.as_bytes().as_ptr() as *const i8,
            Preset::Short => SHORT.as_bytes().as_ptr() as *const i8,
            Preset::Sr => SR.as_bytes().as_ptr() as *const i8,
            Preset::Splice => SPLICE.as_bytes().as_ptr() as *const i8,
            Preset::Cdna => CDNA.as_bytes().as_ptr() as *const i8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Aligner {
    idxopt: mm_idxopt_t,
    mapopt: mm_mapopt_t,
    idx: Option<*mut mm_idx_t>,
    idx_reader: Option<*mut mm_idx_reader_t>,
    threads: usize,
    /* TODO: Goals for better ergonomics...

    // mm_idx_opt
    pub k: u16,
    pub w: u16,
    pub idxflag: u16,
    pub bucket_bits: u16,
    pub mini_batch_size_idx: i64, // Renamed from mini_batch_size
    pub batch_size: u64,

    // mapopt
    pub mapflag: i64,
    pub seed: i32,
    pub sdust_threshold: i32, // Renamed from sdust_thres
    pub max_qlen: i32,
    pub bw: i32,
    pub bw_long: i32,
    pub max_gap: i32,
    pub max_gap_ref: i32,
    pub max_frag_len: i32,
    pub max_chain_skip: i32,
    pub max_chain_iter: i32,
    pub min_cnt: i32,
    pub min_chain_score: i32,
    pub chain_gap_scale: f32,
    pub chain_skip_scale: f32,
    pub rmq_size_cap: i32,
    pub rmq_inner_dist: i32,
    pub rmq_rescue_size: i32,
    pub rmq_rescue_ratio: f32,
    pub mask_level: f32,
    pub mask_len: i32,
    pub pri_ratio: f32,
    pub best_n: i32,
    pub alt_drop: f32,
    pub a: i32,
    pub b: i32,
    pub q: i32,
    pub e: i32,
    pub q2: i32,
    pub e2: i32,
    pub sc_ambi: i32,
    pub noncan: i32,
    pub junc_bonus: i32,
    pub zdrop: i32,
    pub zdrop_inv: i32,
    pub end_bonus: i32,
    pub min_dp_max: i32,
    pub min_ksw_len: i32,
    pub anchor_ext_len: i32,
    pub anchor_ext_shift: i32,
    pub max_clip_ratio: f32,
    pub rank_min_len: i32,
    pub rank_frac: f32,
    pub pe_ori: i32,
    pub pe_bonus: i32,
    pub mid_occ_frac: f32,
    pub q_occ_frac: f32,
    pub min_mid_occ: i32,
    pub max_mid_occ: i32,
    pub mid_occ: i32,
    pub max_occ: i32,
    pub max_max_occ: i32,
    pub occ_dist: i32,
    pub mini_batch_size_map: i64, // Renamed from mini_batch_size
    pub max_sw_mat: i64,
    pub cap_kalloc: i64,
    pub split_prefix: Vec<u8>,
    */
}

impl Default for Aligner {
    fn default() -> Self {
        let mut mm_idxopt = MaybeUninit::uninit();
        let mut mm_mapopt = MaybeUninit::uninit();

        unsafe { mm_set_opt(std::ptr::null(), mm_idxopt.as_mut_ptr(), mm_mapopt.as_mut_ptr()) };
        Self {
            idxopt: unsafe { mm_idxopt.assume_init() },
            mapopt: unsafe { mm_mapopt.assume_init() },
            threads: 1,
            idx: None,
            idx_reader: None,
        }
    }
}

impl Aligner {
    pub fn with_preset(preset: Preset) -> Self {
        let mut mm_idxopt = MaybeUninit::uninit();
        let mut mm_mapopt = MaybeUninit::uninit();

        #[cfg(test)]
        println!("Preset: {:#?}", preset);

        unsafe {
            mm_set_opt(std::ptr::null(), mm_idxopt.as_mut_ptr(), mm_mapopt.as_mut_ptr());
            mm_set_opt(
                preset.into(),
                mm_idxopt.as_mut_ptr(),
                mm_mapopt.as_mut_ptr(),
            )
        };

        Self {
            idxopt: unsafe { mm_idxopt.assume_init() },
            mapopt: unsafe { mm_mapopt.assume_init() },
            ..Default::default()
        }
    }

    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_index() {
        // Index, but instead pass output as None. Placeholder
        todo!();
    }

    pub fn with_named_index(
        mut self,
        path: &Path,
        output: Option<&str>,
    ) -> Result<Self, &'static str> {
        let path = match path.to_str() {
            Some(path) => path,
            None => return Err("Invalid path"),
        };

        let path = match std::ffi::CString::new(path) {
            Ok(path) => path,
            Err(_) => return Err("Invalid path"),
        };

        let output = match output {
            Some(output) => match std::ffi::CString::new(output) {
                Ok(output) => output,
                Err(_) => return Err("Invalid output"),
            },
            None => std::ffi::CString::new(Vec::new()).unwrap(),
        };

        let idx_reader = MaybeUninit::new(unsafe {
            mm_idx_reader_open(
                path.as_ptr(), 
                &self.idxopt, 
                output.as_ptr())
        });

        unsafe {
            if idx_reader.assume_init().is_null() {
                return Err("Failed to create index reader - File not found?");
            }
        }

        self.idx_reader = Some(unsafe { idx_reader.assume_init() });

        let idx = MaybeUninit::new(unsafe {
            mm_idx_reader_read(self.idx_reader.unwrap(), self.threads as libc::c_int)
        });

        self.idx = Some(unsafe { idx.assume_init() });

        Ok(self)
    }

    pub fn with_seq(mut self, seq: &[u8]) -> Result<Self, &'static str> {
        let seq = match std::ffi::CString::new(seq) {
            Ok(seq) => seq,
            Err(_) => return Err("Invalid sequence"),
        };

        todo!();

        //let idx = MaybeUninit::new(unsafe {
        /*mm_idx_str(
            self.idx_opt.w,
            self.idx_opt.k,
            self.idx_opt.flag & 1,
            self.idx_opt.bucket_bits,
            str.encode(seq),
            len(seq),
        )*/
        //});

        //self.idx = Some(idx);

        Ok(self)
    }

    pub fn has_index(&self) -> bool {
        self.idx.is_some()
    }
}

impl Drop for Aligner {
    fn drop(&mut self) {
        if self.idx.is_some() {
            unsafe { mm_idx_destroy(self.idx.unwrap()) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::MaybeUninit;

    #[test]
    fn does_it_work() {
        let mut mm_idxopt = MaybeUninit::uninit();
        let mut mm_mapopt = MaybeUninit::uninit();

        unsafe { 
            mm_set_opt(&0, mm_idxopt.as_mut_ptr(), mm_mapopt.as_mut_ptr()) 
        };
    }

    #[test]
    fn create_index_file_missing() {
        let result = Aligner::with_preset(Preset::MapOnt)
            .with_threads(1)
            .with_named_index(Path::new("test_data/test.fa_FILE_NOT_FOUND"), 
            Some("test_FILE_NOT_FOUND.mmi"));
        assert!(result.is_err());
    }

    #[test]
    fn create_index() {
        let mut aligner = Aligner::with_preset(Preset::MapOnt)
            .with_threads(1);

        println!("{}", aligner.idxopt.w);

        assert!(aligner.idxopt.w == 10);
        
        aligner = aligner.with_named_index(Path::new("test_data/test_data.fasta"),
            Some("test.mmi"))
            .unwrap();
    }
}

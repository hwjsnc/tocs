use super::{parse, serialize};

fn compare_data(expected: &[u8], actual: &[u8]) {
    for i in 0..std::cmp::max(expected.len(), actual.len()) {
        let x1 = expected.get(i);
        let x2 = actual.get(i);
        if x1 != x2 {
            panic!("results differ, first difference at {i} (0x{i:x}). expected {x1:?}, found {x2:?}");
        }
    }
}

#[test]
fn test_battle_scripts_reverie() {
    let dir = "private-test-data/cs5/nisa-pc-1.0.6/scripts/battle/dat_en/";
    //let dir = "private-test-data/cs5/nisa-pc-1.0.6/scripts/scena/dat_en/";
    //let dir = "private-test-data/cs5/nisa-pc-1.0.6/scripts/ani/dat_en/";
    for dirent in std::fs::read_dir(dir).unwrap() {
        let path = dirent.unwrap().path();
        println!("{}", path.display());
        let data = std::fs::read(path).unwrap();
        let aldat = parse(data.as_slice()).unwrap();
        //println!("{:?}", aldat);
        let roundtripped = serialize(&aldat).unwrap();
        std::fs::write("/tmp/foo.tmp", &roundtripped).unwrap();
        compare_data(&data, &roundtripped);
    }
}

#[test]
fn test_tmp() {
    /*
    let dir = "private-test-data/cs5/nisa-pc-1.0.6/scripts/battle/dat_en/";
    let skip = [
        "btl1000.dat",
        "btl1001.dat",
        "btlwin.dat",
        "btl_00_01_02.dat",
        "btl_A1_11_01.dat",
        "btl_A4_20_01.dat",
        "btl_TU_00_00_00.dat",
        "btl_TU_00_02_00.dat",
        "btl_TU_00_03_00.dat",
        "btl_TU_A1_03_00.dat",
        "btl_TU_B1_06_00.dat",
        "btlsys.dat",
    ];
    for dirent in std::fs::read_dir(dir).unwrap() {
        let path = dirent.unwrap().path();
        if skip.iter().any(|f| path.ends_with(f)) {
            println!("{}", path.display());
            let data = std::fs::read(path).unwrap();
            assert!(parse(data.as_slice()).is_none());
        }
    }
    todo!()
    */
}

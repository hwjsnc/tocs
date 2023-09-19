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
fn test_scripts_reverie() {
    let roundtrip_fails: std::collections::BTreeSet<&'static str> = [
        "ani/dat_en/chr003_mg16.dat",
        "ani/dat_en/chr970_c00.dat",
        "ani/dat_en/mon000s.dat",
        "ani/dat_en/mon027_c00.dat",
        "ani/dat_en/mon037_c00.dat",
        "ani/dat_en/mon042_c00.dat",
        "ani/dat_en/mon042_c01.dat",
        "ani/dat_en/mon046_c00.dat",
        "ani/dat_en/mon093.dat",
        "ani/dat_en/mon426.dat",
        "ani/dat_en/mon_171.dat",
        "ani/dat_en/mon_template.dat",
        "ani/dat_en/npcx00.dat",
        "ani/dat_en/npcx02.dat",
        "ani/dat_en/npcx03.dat",
        "ani/dat_en/npcx04.dat",
        "ani/dat_en/ply000.dat",
        "ani/dat_en/rob030.dat",
        "ani/dat_en/chr_enemy_template.dat",
        "ani/dat_en/rob013_c00.dat",
        "ani/dat_en/ply001.dat",
        "scena/dat_en/a0106.dat",
    ]
    .into_iter()
    .collect();

    let base = std::path::Path::new("private-test-data/cs5/nisa-pc-1.0.6/scripts/");
    for script_folder_dirent in std::fs::read_dir(base).unwrap() {
        let dir = script_folder_dirent.unwrap().path().join("dat_en");
        for dirent in std::fs::read_dir(dir).unwrap() {
            let path = dirent.unwrap().path();
            let relative_path = path.strip_prefix(base).unwrap().to_str().unwrap();
            if roundtrip_fails.contains(relative_path) {
                println!("SKIPPING {relative_path}");
                continue;
            }
            println!("{relative_path}");
            let data = std::fs::read(path).unwrap();
            let aldat = parse(data.as_slice()).unwrap();
            //println!("{:?}", aldat);
            let roundtripped = serialize(&aldat).unwrap();
            //std::fs::write("/tmp/foo.tmp", &roundtripped).unwrap();
            compare_data(&data, &roundtripped);
        }
    }

    for relative_path in roundtrip_fails.iter().copied() {
        let path = base.join(relative_path);
        println!("{relative_path}");
        let data = std::fs::read(&path).unwrap();
        let aldat = parse(data.as_slice()).unwrap();
        //println!("{:?}", aldat);
        let roundtripped = serialize(&aldat).unwrap();
        assert!(data != roundtripped);
    }
}

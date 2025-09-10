use std::path::Path;

fn main() {
    // ให้ build.rs รันใหม่เมื่อไฟล์ C++/header เปลี่ยน
    println!("cargo:rerun-if-changed=native/spfresh_c_api.cc");
    println!("cargo:rerun-if-changed=native/spfresh_c_api.h");

    // คอมไพล์ C++ wrapper => libspfresh_c_api.a
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .file("native/spfresh_c_api.cc")
        .include("third_party/SPFresh")
        .include("third_party/SPFresh/AnnService")
        .include("third_party/SPFresh/Common");
    build.compile("spfresh_c_api");

    // ที่ ๆ อาจมีไฟล์ .a ของ SPTAG อยู่
    let candidates = [
        "/usr/local/lib",
        "third_party/SPFresh/build",
        "third_party/SPFresh/build/lib",
        "third_party/SPFresh/build/Release",
        "third_party/SPFresh/build/AnnService",
        "third_party/SPFresh/build/AnnService/Release",
        "third_party/SPFresh/build/Common",
        "third_party/SPFresh/build/Common/Release",
        "third_party/SPFresh/Release",
    ];
    for dir in candidates {
        println!("cargo:rustc-link-search=native={dir}");
    }

    // เลือกชื่อ lib SPTAG ตามไฟล์ที่มีจริง
    let mut linked_sptag = false;
    for dir in &candidates {
        let path_static = format!("{dir}/libSPTAGLibStatic.a");
        let path_plain  = format!("{dir}/libSPTAGLib.a");
        if Path::new(&path_static).exists() {
            println!("cargo:rustc-link-lib=static=SPTAGLibStatic");
            linked_sptag = true;
            break;
        }
        if Path::new(&path_plain).exists() {
            println!("cargo:rustc-link-lib=static=SPTAGLib");
            linked_sptag = true;
            break;
        }
    }
    if !linked_sptag {
        println!("cargo:warning=Could not find libSPTAGLibStatic.a or libSPTAGLib.a in known locations");
    }

    // DistanceUtils
    let mut linked_du = false;
    for dir in &candidates {
        let path_du = format!("{dir}/libDistanceUtils.a");
        if Path::new(&path_du).exists() {
            println!("cargo:rustc-link-lib=static=DistanceUtils");
            linked_du = true;
            break;
        }
    }
    if !linked_du {
        println!("cargo:warning=Could not find libDistanceUtils.a in known locations");
    }

    // ไลบรารีระบบ/third-party
    println!("cargo:rustc-link-lib=rocksdb");
    println!("cargo:rustc-link-lib=snappy");
    println!("cargo:rustc-link-lib=gflags");
    println!("cargo:rustc-link-lib=tbb");
    println!("cargo:rustc-link-lib=isal");
    println!("cargo:rustc-link-lib=jemalloc");
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=ssl");
    println!("cargo:rustc-link-lib=crypto");
}

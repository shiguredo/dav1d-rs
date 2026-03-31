use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

// 依存ライブラリの名前
const LIB_NAME: &str = "dav1d";

// シンボル書き換え用のプレフィックス
//
// prebuilt で配布する際、他のライブラリが同じ dav1d シンボル (dav1d_open, dav1d_close 等) を
// 使っていると衝突する。この定数のプレフィックスを付けることで回避する。
//
// 変換例:
//   dav1d_open      → shiguredo_dav1d_open  (dav1d_ を shiguredo_dav1d_ に置換)
//   bitfn_clz       → shiguredo_dav1d_bitfn_clz (内部シンボルは単純にプレフィックス付与)
const SYMBOL_PREFIX: &str = "shiguredo_dav1d";

fn main() {
    // Cargo.toml か build.rs が更新されたら、依存ライブラリを再ビルドする
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_SOURCE_BUILD");
    println!("cargo::rerun-if-env-changed=DAV1D_TARGET");

    // 各種変数やビルドディレクトリのセットアップ
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("infallible"));
    let output_metadata_path = out_dir.join("metadata.rs");
    let output_bindings_path = out_dir.join("bindings.rs");

    // 各種メタデータを書き込む
    let (git_url, version) = get_git_url_and_version();
    fs::write(
        output_metadata_path,
        format!(
            concat!(
                "pub const BUILD_METADATA_REPOSITORY: &str={:?};\n",
                "pub const BUILD_METADATA_VERSION: &str={:?};\n",
            ),
            git_url, version
        ),
    )
    .expect("failed to write metadata file");

    if env::var("DOCS_RS").is_ok() {
        // Docs.rs 向けのビルドでは git clone ができないので build.rs の処理はスキップして、
        // 代わりに、ドキュメント生成時に最低限必要な構造体だけをダミーで出力している。
        //
        // シンボル書き換えもスキップされる（ビルド自体が行われないため）。
        //
        // See also: https://docs.rs/about/builds
        fs::write(
            output_bindings_path,
            r#"
// docs.rs 向けダミー定義
pub struct Dav1dContext;
pub struct Dav1dPicture;
pub struct Dav1dSettings;
pub struct Dav1dData;
pub struct Dav1dSequenceHeader;
pub struct Dav1dDataProps;

pub type Dav1dPixelLayout = u32;
pub const Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I400: u32 = 0;
pub const Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I420: u32 = 1;
pub const Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I422: u32 = 2;
pub const Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I444: u32 = 3;

pub type Dav1dColorPrimaries = u32;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT709: u32 = 1;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_UNKNOWN: u32 = 2;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT470M: u32 = 4;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT470BG: u32 = 5;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT601: u32 = 6;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE240: u32 = 7;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_FILM: u32 = 8;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT2020: u32 = 9;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_XYZ: u32 = 10;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE431: u32 = 11;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE432: u32 = 12;
pub const Dav1dColorPrimaries_DAV1D_COLOR_PRI_EBU3213: u32 = 22;

pub type Dav1dTransferCharacteristics = u32;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT709: u32 = 1;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_UNKNOWN: u32 = 2;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT470M: u32 = 4;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT470BG: u32 = 5;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT601: u32 = 6;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE240: u32 = 7;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_LINEAR: u32 = 8;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_LOG100: u32 = 9;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_LOG100_SQRT10: u32 = 10;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_IEC61966: u32 = 11;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT1361: u32 = 12;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_SRGB: u32 = 13;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT2020_10BIT: u32 = 14;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_BT2020_12BIT: u32 = 15;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE2084: u32 = 16;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE428: u32 = 17;
pub const Dav1dTransferCharacteristics_DAV1D_TRC_HLG: u32 = 18;

pub type Dav1dMatrixCoefficients = u32;
pub const Dav1dMatrixCoefficients_DAV1D_MC_IDENTITY: u32 = 0;
pub const Dav1dMatrixCoefficients_DAV1D_MC_BT709: u32 = 1;
pub const Dav1dMatrixCoefficients_DAV1D_MC_UNKNOWN: u32 = 2;
pub const Dav1dMatrixCoefficients_DAV1D_MC_FCC: u32 = 4;
pub const Dav1dMatrixCoefficients_DAV1D_MC_BT470BG: u32 = 5;
pub const Dav1dMatrixCoefficients_DAV1D_MC_BT601: u32 = 6;
pub const Dav1dMatrixCoefficients_DAV1D_MC_SMPTE240: u32 = 7;
pub const Dav1dMatrixCoefficients_DAV1D_MC_SMPTE_YCGCO: u32 = 8;
pub const Dav1dMatrixCoefficients_DAV1D_MC_BT2020_NCL: u32 = 9;
pub const Dav1dMatrixCoefficients_DAV1D_MC_BT2020_CL: u32 = 10;
pub const Dav1dMatrixCoefficients_DAV1D_MC_SMPTE2085: u32 = 11;
pub const Dav1dMatrixCoefficients_DAV1D_MC_CHROMAT_NCL: u32 = 12;
pub const Dav1dMatrixCoefficients_DAV1D_MC_CHROMAT_CL: u32 = 13;
pub const Dav1dMatrixCoefficients_DAV1D_MC_ICTCP: u32 = 14;

pub type Dav1dChromaSamplePosition = u32;
pub const Dav1dChromaSamplePosition_DAV1D_CHR_UNKNOWN: u32 = 0;
pub const Dav1dChromaSamplePosition_DAV1D_CHR_VERTICAL: u32 = 1;
pub const Dav1dChromaSamplePosition_DAV1D_CHR_COLOCATED: u32 = 2;

pub type Dav1dFrameType = u32;
pub const Dav1dFrameType_DAV1D_FRAME_TYPE_KEY: u32 = 0;
pub const Dav1dFrameType_DAV1D_FRAME_TYPE_INTER: u32 = 1;
pub const Dav1dFrameType_DAV1D_FRAME_TYPE_INTRA: u32 = 2;
pub const Dav1dFrameType_DAV1D_FRAME_TYPE_SWITCH: u32 = 3;

pub type Dav1dInloopFilterType = u32;
pub const Dav1dInloopFilterType_DAV1D_INLOOPFILTER_NONE: u32 = 0;
pub const Dav1dInloopFilterType_DAV1D_INLOOPFILTER_DEBLOCK: u32 = 1;
pub const Dav1dInloopFilterType_DAV1D_INLOOPFILTER_CDEF: u32 = 2;
pub const Dav1dInloopFilterType_DAV1D_INLOOPFILTER_RESTORATION: u32 = 4;
pub const Dav1dInloopFilterType_DAV1D_INLOOPFILTER_ALL: u32 = 7;

pub type Dav1dDecodeFrameType = u32;
pub const Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_ALL: u32 = 0;
pub const Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_REFERENCE: u32 = 1;
pub const Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_INTRA: u32 = 2;
pub const Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_KEY: u32 = 3;

pub type Dav1dEventFlags = u32;
pub const Dav1dEventFlags_DAV1D_EVENT_FLAG_NEW_SEQUENCE: u32 = 1;
pub const Dav1dEventFlags_DAV1D_EVENT_FLAG_NEW_OP_PARAMS_INFO: u32 = 2;

pub const EAGAIN: u32 = 35;
pub const ENOMEM: u32 = 12;
"#,
        )
        .expect("write file error");
        return;
    }

    let output_lib_dir = if should_use_prebuilt() {
        download_prebuilt(&out_dir)
    } else {
        build_from_source(&out_dir, &output_bindings_path)
    };

    println!("cargo::rustc-link-search={}", output_lib_dir.display());
    println!("cargo::rustc-link-lib=static={LIB_NAME}");
}

// source-build feature が有効でなければ prebuilt を使う
fn should_use_prebuilt() -> bool {
    env::var("CARGO_FEATURE_SOURCE_BUILD").is_err()
}

// prebuilt バイナリをダウンロードして展開する
fn download_prebuilt(out_dir: &Path) -> PathBuf {
    let target = get_target_platform();
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION is not set");
    let base_url = format!(
        "https://github.com/shiguredo/dav1d-rs/releases/download/{}",
        version
    );
    let archive_name = format!("dav1d-{}.tar.gz", target);
    let archive_url = format!("{}/{}", base_url, archive_name);
    let sha256_url = format!("{}/{}.sha256", base_url, archive_name);

    let archive_path = out_dir.join("prebuilt.tar.gz");
    let sha256_path = out_dir.join("prebuilt.sha256");
    let prebuilt_dir = out_dir.join("prebuilt");
    fs::create_dir_all(&prebuilt_dir).expect("failed to create prebuilt directory");

    // curl でアーカイブをダウンロード
    eprintln!("prebuilt ライブラリをダウンロード中: {}", archive_url);
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&archive_path)
        .arg(&archive_url)
        .status()
        .expect("failed to execute curl. Ensure curl is installed");
    if !status.success() {
        panic!("failed to download prebuilt library: {}", archive_url);
    }

    // curl で SHA256 チェックサムをダウンロード
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&sha256_path)
        .arg(&sha256_url)
        .status()
        .expect("failed to execute curl");
    if !status.success() {
        panic!("failed to download SHA256 checksum: {}", sha256_url);
    }

    // SHA256 を検証
    verify_sha256(&archive_path, &sha256_path);

    // tar で展開
    let status = Command::new("tar")
        .args(["xzf"])
        .arg(&archive_path)
        .arg("-C")
        .arg(&prebuilt_dir)
        .status()
        .expect("failed to execute tar. Ensure tar is installed");
    if !status.success() {
        panic!("failed to extract prebuilt archive");
    }

    // ライブラリファイルを OUT_DIR/lib/ にコピー
    //
    // prebuilt バイナリはシンボル書き換え済み・bindings.rs は #[link_name] 付きで
    // 配布されているため、コピーするだけでよい。
    let lib_dir = out_dir.join("lib");
    fs::create_dir_all(&lib_dir).expect("failed to create lib directory");
    fs::copy(
        prebuilt_dir.join("lib").join("libdav1d.a"),
        lib_dir.join("libdav1d.a"),
    )
    .expect("failed to copy libdav1d.a");

    // bindings.rs を OUT_DIR/ にコピー
    fs::copy(
        prebuilt_dir.join("bindings.rs"),
        out_dir.join("bindings.rs"),
    )
    .expect("failed to copy bindings.rs");

    lib_dir
}

// SHA256 チェックサムを検証する
fn verify_sha256(file_path: &Path, sha256_path: &Path) {
    let expected = fs::read_to_string(sha256_path)
        .expect("failed to read SHA256 checksum file")
        .split_whitespace()
        .next()
        .expect("SHA256 checksum file is empty")
        .to_lowercase();

    let actual = compute_sha256(file_path);
    if actual != expected {
        panic!(
            "SHA256 checksum mismatch:\n  expected: {}\n  actual:   {}",
            expected, actual
        );
    }
    eprintln!("SHA256 checksum verified: {}", actual);
}

// ファイルの SHA256 ハッシュを計算する
fn compute_sha256(path: &Path) -> String {
    let output = if cfg!(target_os = "macos") {
        // macOS: shasum を使用
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()
            .expect("failed to execute shasum. Ensure shasum is installed")
    } else if cfg!(target_os = "windows") {
        // Windows: certutil を使用
        Command::new("certutil")
            .args(["-hashfile"])
            .arg(path)
            .arg("SHA256")
            .output()
            .expect("failed to execute certutil")
    } else {
        // Linux: sha256sum を使用
        Command::new("sha256sum")
            .arg(path)
            .output()
            .expect("failed to execute sha256sum. Ensure coreutils is installed")
    };

    if !output.status.success() {
        panic!("failed to compute SHA256 checksum");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if cfg!(target_os = "windows") {
        // certutil 出力形式:
        // SHA256 hash of <file>:
        // <hash>
        // CertUtil: -hashfile command completed successfully.
        stdout
            .lines()
            .nth(1)
            .expect("unexpected certutil output format")
            .trim()
            .to_lowercase()
    } else {
        // shasum / sha256sum 出力形式: <hash>  <filename>
        stdout
            .split_whitespace()
            .next()
            .expect("unexpected shasum/sha256sum output format")
            .to_lowercase()
    }
}

// ソースからビルドする
fn build_from_source(out_dir: &Path, output_bindings_path: &Path) -> PathBuf {
    let out_build_dir = out_dir.join("build/");
    let src_dir = out_build_dir.join(LIB_NAME);
    let src_build_dir = src_dir.join("build/");
    let input_header_path = src_dir.join("include/dav1d/dav1d.h");
    let output_lib_dir = src_build_dir.join("src/");
    let _ = fs::remove_dir_all(&out_build_dir);
    fs::create_dir(&out_build_dir).expect("failed to create build directory");

    // 依存ライブラリのリポジトリを取得する
    git_clone_external_lib(&out_build_dir);

    // 依存ライブラリをビルドする
    fs::create_dir(&src_build_dir).expect("failed to create build directory");

    // Windows MSVC ターゲットでは --vsenv を指定して meson に Visual Studio 環境を
    // 自動検出させる。PATH に MinGW GCC がある場合でも MSVC が優先される。
    // --vsenv は meson 0.60.0 以降で利用可能。
    let is_msvc = env::var("CARGO_CFG_TARGET_ENV")
        .map(|v| v == "msvc")
        .unwrap_or(false);

    let mut meson_cmd = Command::new("meson");
    meson_cmd.arg("setup").arg("--default-library=static");

    if is_msvc {
        meson_cmd.arg("--vsenv");
    }

    meson_cmd.arg("..").current_dir(&src_build_dir);

    let success = meson_cmd.status().is_ok_and(|status| status.success());
    if !success {
        panic!("[meson] failed to build {LIB_NAME}");
    }

    // Windows MSVC では --vsenv で検出した VS 環境を ninja に引き継ぐために
    // meson compile を使う必要がある。ninja を直接呼ぶと VS 環境が失われる。
    let success = if is_msvc {
        Command::new("meson")
            .args(["compile", "-C"])
            .arg(&src_build_dir)
            .status()
            .is_ok_and(|status| status.success())
    } else {
        Command::new("ninja")
            .current_dir(&src_build_dir)
            .status()
            .is_ok_and(|status| status.success())
    };
    if !success {
        panic!("[build] failed to build {LIB_NAME}");
    }

    // 静的ライブラリのシンボルを書き換える
    let callbacks = rewrite_symbols(&output_lib_dir, out_dir);

    // バインディングを生成する
    //
    // parse_callbacks にシンボル書き換え用の ParseCallbacks を渡すことで、
    // 生成されるバインディングに #[link_name = "書き換え後のシンボル名"] が自動付与される。
    bindgen::Builder::default()
        .header(input_header_path.to_str().expect("invalid header path"))
        .parse_callbacks(Box::new(callbacks))
        .generate()
        .expect("failed to generate bindings")
        .write_to_file(output_bindings_path)
        .expect("failed to write bindings");

    output_lib_dir
}

// --- シンボル書き換え ---
//
// 他のライブラリとのシンボル衝突を回避するため、静的ライブラリ内の全シンボルに
// プレフィックスを付与する仕組み。
//
// llvm-nm / llvm-objcopy は rustup の llvm-tools コンポーネントに含まれるものを使用する。
// rust-toolchain.toml に components = ["llvm-tools"] の記載が必要。
//
// プラットフォームごとのシンボル形式の違い:
//   - macOS (Mach-O): シンボル先頭に `_` が付く (例: _dav1d_open)
//   - Linux (ELF): 先頭 `_` なし (例: dav1d_open)
//   - Windows x64 (COFF): 先頭 `_` なし (例: dav1d_open)
//
// bindgen の generated_link_name_override は返した文字列に \u{1} プレフィックスを
// 自動付加する。\u{1} はコンパイラに「この名前をそのまま使え（マングリングするな）」と
// 指示するため、プラットフォーム固有のシンボル名（macOS なら _shiguredo_dav1d_open）を
// そのまま返す必要がある。

/// llvm-nm / llvm-objcopy のパスを保持する
struct LlvmTools {
    nm: PathBuf,
    objcopy: PathBuf,
}

/// objcopy 用と bindgen 用の 2 つのリネームマップを保持する
///
/// 2 つのマップが必要な理由:
///   - objcopy_map: ライブラリ内の実シンボル名を書き換えるため、プラットフォーム依存の名前を使う
///   - bindgen_map: Rust コードからリンクする際の名前を指定するため、C シンボル名をキーにする
struct SymbolRenameMaps {
    /// llvm-objcopy の --redefine-syms 用マップ
    ///
    /// キー: 元のシンボル名 (例: macOS なら _dav1d_open、Linux なら dav1d_open)
    /// 値: 書き換え後のシンボル名 (例: macOS なら _shiguredo_dav1d_open)
    objcopy_map: HashMap<String, String>,

    /// bindgen の #[link_name] 用マップ
    ///
    /// キー: C シンボル名 (プラットフォーム非依存、例: dav1d_open)
    /// 値: 書き換え後のシンボル名 (プラットフォーム依存、例: macOS なら _shiguredo_dav1d_open)
    ///
    /// bindgen は \u{1} プレフィックスを付加してマングリングを抑制するため、
    /// 値にはプラットフォーム固有のシンボル名を格納する必要がある。
    bindgen_map: HashMap<String, String>,
}

/// bindgen の ParseCallbacks 実装
///
/// バインディング生成時に、書き換え後のシンボル名を `#[link_name = "..."]` として付与する。
/// これにより lib.rs 側のコード変更なしでシンボル書き換えが透過的に動作する。
#[derive(Debug)]
struct SymbolLinkNameCallbacks {
    /// C シンボル名 → 書き換え後シンボル名のマップ
    rename_map: HashMap<String, String>,
}

impl bindgen::callbacks::ParseCallbacks for SymbolLinkNameCallbacks {
    /// bindgen がバインディングを生成する際に呼ばれるコールバック
    ///
    /// 戻り値が Some の場合、bindgen は #[link_name = "\u{1}<戻り値>"] を生成する。
    /// \u{1} プレフィックスによりコンパイラのシンボルマングリングが抑制されるため、
    /// 戻り値にはプラットフォーム固有のシンボル名を返す必要がある。
    fn generated_link_name_override(
        &self,
        item_info: bindgen::callbacks::ItemInfo<'_>,
    ) -> Option<String> {
        self.rename_map.get(item_info.name).cloned()
    }
}

/// 静的ライブラリのシンボルを書き換え、bindgen 用の ParseCallbacks を返す
///
/// 処理の流れ:
///   1. rustup の sysroot から llvm-nm / llvm-objcopy を探す
///   2. llvm-nm で静的ライブラリの定義済み外部シンボルを収集する
///   3. 収集したシンボルに対してリネームマップを生成する
///   4. マップファイルを書き出し、llvm-objcopy でライブラリ内のシンボルを書き換える
///   5. bindgen 用の ParseCallbacks を返す
fn rewrite_symbols(lib_dir: &Path, out_dir: &Path) -> SymbolLinkNameCallbacks {
    let tools = discover_llvm_tools();
    let lib_path = find_static_library(lib_dir);

    // macOS の Mach-O ではシンボル先頭に `_` が付くため、
    // プラットフォーム判定してリネームマップの生成時に考慮する
    let is_macos = env::var("CARGO_CFG_TARGET_OS")
        .map(|v| v == "macos")
        .unwrap_or(false);

    // シンボル名の変換ルール
    //
    // dav1d_ プレフィックスを持つシンボル (公開 API) は dav1d_ を SYMBOL_PREFIX_ に置換する。
    //   例: dav1d_open → shiguredo_dav1d_open
    //
    // それ以外のシンボル (bitfn_*, msac_* 等の内部シンボル) は先頭に SYMBOL_PREFIX_ を付与する。
    //   例: bitfn_clz → shiguredo_dav1d_bitfn_clz
    let rename_symbol = |name: &str| -> Option<String> {
        if let Some(rest) = name.strip_prefix("dav1d_") {
            Some(format!("{SYMBOL_PREFIX}_{rest}"))
        } else {
            Some(format!("{SYMBOL_PREFIX}_{name}"))
        }
    };

    // 全定義済み外部シンボルを収集してリネームマップを生成する
    let symbols = collect_defined_external_symbols(&tools.nm, &lib_path);
    let maps = build_symbol_rename_maps(&symbols, is_macos, &rename_symbol);

    // マップファイルを書き出してシンボルを書き換える
    let map_file = out_dir.join("symbol_rename_map.txt");
    write_objcopy_rename_map(&maps.objcopy_map, &map_file);
    rewrite_archive_symbols(&tools.objcopy, &lib_path, &map_file);

    SymbolLinkNameCallbacks {
        rename_map: maps.bindgen_map,
    }
}

/// 静的ライブラリのパスを探す
///
/// 全プラットフォームで libdav1d.a に統一している。
/// Windows MSVC では meson が dav1d.lib を生成するため、libdav1d.a にリネームする。
fn find_static_library(lib_dir: &Path) -> PathBuf {
    let lib_path = lib_dir.join("libdav1d.a");
    if lib_path.exists() {
        return lib_path;
    }

    // Windows MSVC では meson が dav1d.lib を生成するため libdav1d.a にリネームする
    let msvc_lib_path = lib_dir.join("dav1d.lib");
    if msvc_lib_path.exists() {
        fs::rename(&msvc_lib_path, &lib_path).expect("failed to rename dav1d.lib to libdav1d.a");
        return lib_path;
    }

    panic!("static library not found in {}", lib_dir.display());
}

/// rustc --print sysroot の結果を取得する
///
/// llvm-tools は rustup が管理する sysroot 配下にインストールされるため、
/// sysroot のパスを取得して llvm-nm / llvm-objcopy の探索に使用する。
fn get_rustc_sysroot() -> PathBuf {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .expect("failed to run rustc --print sysroot");
    if !output.status.success() {
        panic!("rustc --print sysroot failed");
    }
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("invalid UTF-8")
            .trim(),
    )
}

/// Windows 対応の実行ファイル名を生成する
///
/// Windows では実行ファイルに .exe 拡張子が必要。
fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// rustup の sysroot から llvm-nm / llvm-objcopy を探す
///
/// llvm-tools コンポーネントのバイナリは以下のパスに配置される:
///   <sysroot>/lib/rustlib/<host>/bin/llvm-nm
///   <sysroot>/lib/rustlib/<host>/bin/llvm-objcopy
///
/// rust-toolchain.toml に llvm-tools コンポーネントの記載が必要。
///
/// llvm-nm / llvm-objcopy はホスト上で実行するツールなので、クロスコンパイル時は
/// TARGET ではなく HOST のパスから探す必要がある。
fn discover_llvm_tools() -> LlvmTools {
    let sysroot = get_rustc_sysroot();
    // llvm-tools はホスト上で動作するため HOST を使う。
    // クロスコンパイル時に TARGET を使うと、ホスト側にインストールされた
    // llvm-tools が見つからない。
    let host = env::var("HOST").expect("HOST environment variable not set");
    let tools_dir = sysroot.join("lib/rustlib").join(host).join("bin");

    let nm = tools_dir.join(exe_name("llvm-nm"));
    let objcopy = tools_dir.join(exe_name("llvm-objcopy"));

    if !nm.exists() {
        panic!(
            "llvm-nm not found at {}. Run: rustup component add llvm-tools",
            nm.display()
        );
    }
    if !objcopy.exists() {
        panic!(
            "llvm-objcopy not found at {}. Run: rustup component add llvm-tools",
            objcopy.display()
        );
    }

    LlvmTools { nm, objcopy }
}

/// llvm-nm で静的ライブラリから定義済み外部シンボルを収集する
///
/// llvm-nm のオプション:
///   --defined-only: 定義済みシンボルのみ (未定義シンボルを除外)
///   --extern-only: 外部シンボルのみ (ローカルシンボルを除外)
///   --format=just-symbols: シンボル名のみ出力 (アドレスやタイプを省略)
///
/// 出力にはオブジェクトファイル名 (例: dav1d.c.o:) も含まれるため、
/// is_c_identifier() でフィルタリングして純粋なシンボル名のみを抽出する。
fn collect_defined_external_symbols(nm_path: &Path, lib_path: &Path) -> Vec<String> {
    let output = Command::new(nm_path)
        .arg("--defined-only")
        .arg("--extern-only")
        .arg("--format=just-symbols")
        .arg(lib_path)
        .output()
        .expect("failed to run llvm-nm");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("llvm-nm failed: {stderr}");
    }

    let stdout = String::from_utf8(output.stdout).expect("llvm-nm output is not valid UTF-8");
    let mut symbols: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|s| !s.is_empty() && is_symbol_name(s))
        .collect();
    symbols.sort();
    symbols.dedup();
    symbols
}

/// シンボル名として有効かどうかを判定する
///
/// llvm-nm の --format=just-symbols 出力にはオブジェクトファイル名 (dav1d.c.o: 等) も
/// 含まれるため、この関数でシンボル名のみをフィルタリングする。
///
/// macOS の Mach-O ではシンボル先頭に `_` が付くため、`_` で始まる文字列も受け入れる。
///
/// NASM が生成する x86_64 向けシンボルには `.` が含まれる場合がある
/// (例: dav1d_cdef_dir_8bpc_avx2.main)。これらもリネーム対象にするため `.` を許可する。
/// オブジェクトファイル名は末尾に `:` が付く (例: dav1d.c.o:) ため、
/// `:` を不許可にすることで区別する。
fn is_symbol_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c == '_' || c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c == '.' || c.is_ascii_alphanumeric())
}

/// objcopy 用と bindgen 用のリネームマップを生成する
///
/// 2 つのマップを生成する理由:
///
/// objcopy_map: ライブラリバイナリ内の実シンボル名を書き換えるためのマップ。
///   macOS では _dav1d_open → _shiguredo_dav1d_open のようにプラットフォーム固有の
///   `_` プレフィックスを含む形で管理する。
///
/// bindgen_map: Rust バインディングの #[link_name] に使うマップ。
///   キーは C シンボル名 (dav1d_open)、値はプラットフォーム固有のシンボル名
///   (_shiguredo_dav1d_open) を格納する。
///   bindgen は generated_link_name_override の戻り値に \u{1} を付加してマングリングを
///   抑制するため、プラットフォーム固有の名前を直接返す必要がある。
fn build_symbol_rename_maps(
    symbols: &[String],
    is_macos: bool,
    rename_symbol: &dyn Fn(&str) -> Option<String>,
) -> SymbolRenameMaps {
    let mut objcopy_map = HashMap::new();
    let mut bindgen_map = HashMap::new();

    for sym in symbols {
        // プラットフォーム固有のプレフィックスを除去して C シンボル名を取得する
        //   macOS: _dav1d_open → dav1d_open
        //   Linux/Windows: dav1d_open → dav1d_open (変化なし)
        let c_name = if is_macos {
            sym.strip_prefix('_').unwrap_or(sym)
        } else {
            sym.as_str()
        };

        if let Some(new_c_name) = rename_symbol(c_name) {
            // objcopy 用: プラットフォーム固有のプレフィックスを再付与する
            //   macOS: shiguredo_dav1d_open → _shiguredo_dav1d_open
            //   Linux/Windows: shiguredo_dav1d_open → shiguredo_dav1d_open (変化なし)
            let new_sym = if is_macos {
                format!("_{new_c_name}")
            } else {
                new_c_name.clone()
            };
            objcopy_map.insert(sym.clone(), new_sym.clone());

            // bindgen 用: generated_link_name_override は \u{1} プレフィックスを付加して
            // シンボル名をそのまま使うため、プラットフォーム固有のシンボル名で管理する
            bindgen_map.insert(c_name.to_string(), new_sym);
        }
    }

    SymbolRenameMaps {
        objcopy_map,
        bindgen_map,
    }
}

/// --redefine-syms 用のマップファイルを書き出す
///
/// ファイル形式は 1 行に "旧シンボル名 新シンボル名" を空白区切りで記述する。
/// llvm-objcopy の --redefine-syms オプションで使用される。
fn write_objcopy_rename_map(map: &HashMap<String, String>, path: &Path) {
    let mut lines: Vec<String> = map
        .iter()
        .map(|(old, new)| format!("{old} {new}"))
        .collect();
    // 出力を決定的にするためソートする
    lines.sort();
    fs::write(path, lines.join("\n")).expect("failed to write symbol rename map");
}

/// llvm-objcopy でアーカイブ内のシンボルを書き換える
///
/// --redefine-syms はマップファイルに従ってシンボル名を一括置換する。
/// ライブラリファイルはインプレースで更新される。
fn rewrite_archive_symbols(objcopy_path: &Path, lib_path: &Path, map_file: &Path) {
    let status = Command::new(objcopy_path)
        .arg("--redefine-syms")
        .arg(map_file)
        .arg(lib_path)
        .status()
        .expect("failed to run llvm-objcopy");
    if !status.success() {
        panic!("llvm-objcopy failed");
    }
}

// --- 既存のヘルパー関数 ---

// CARGO_CFG_TARGET_OS + CARGO_CFG_TARGET_ARCH からプラットフォーム名を生成する
fn get_target_platform() -> String {
    if let Ok(target) = env::var("DAV1D_TARGET") {
        return target;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    match (target_os.as_str(), target_arch.as_str()) {
        ("linux", "x86_64") => format!("{}_x86_64", detect_linux_distro()),
        ("linux", "aarch64") => format!("{}_arm64", detect_linux_distro()),
        ("macos", "aarch64") => "macos_arm64".to_string(),
        ("windows", "x86_64") => "windows_x86_64".to_string(),
        _ => panic!("unsupported target: os={}, arch={}", target_os, target_arch),
    }
}

// /etc/os-release から Ubuntu バージョンを検出する
fn detect_linux_distro() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(version) = line.strip_prefix("VERSION_ID=") {
                let version = version.trim_matches('"');
                match version {
                    "22.04" | "24.04" => return format!("ubuntu-{}", version),
                    _ => {}
                }
            }
        }
    }
    panic!(
        "unsupported Linux distribution. \
         set DAV1D_TARGET environment variable to specify the target explicitly"
    );
}

// 外部ライブラリのリポジトリを git clone する
fn git_clone_external_lib(build_dir: &Path) {
    let (git_url, version) = get_git_url_and_version();
    let repo_dir = build_dir.join(LIB_NAME);

    // shallow clone してタグをチェックアウトする
    //
    // アノテーティッドタグの場合 --branch が使えない git バージョンがあるため、
    // まず --no-checkout で clone してから fetch + checkout する
    let success = Command::new("git")
        .args(["clone", "--depth", "1", "--no-checkout"])
        .arg(&git_url)
        .arg(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to clone {LIB_NAME} repository");
    }

    let success = Command::new("git")
        .args(["fetch", "--depth", "1", "origin", "tag"])
        .arg(&version)
        .current_dir(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to fetch tag {version}");
    }

    let success = Command::new("git")
        .args(["checkout", "FETCH_HEAD"])
        .current_dir(&repo_dir)
        .status()
        .is_ok_and(|status| status.success());
    if !success {
        panic!("failed to checkout tag {version}");
    }
}

// Cargo.toml から依存ライブラリの URL とバージョンタグを取得する
fn get_git_url_and_version() -> (String, String) {
    let cargo_toml = shiguredo_toml::Value::Table(
        shiguredo_toml::from_str(include_str!("Cargo.toml")).expect("failed to parse Cargo.toml"),
    );
    if let Some((Some(url), Some(version))) = cargo_toml
        .get("package")
        .and_then(|v| v.get("metadata"))
        .and_then(|v| v.get("external-dependencies"))
        .and_then(|v| v.get(LIB_NAME))
        .map(|v| {
            (
                v.get("url").and_then(|s| s.as_str()),
                v.get("version").and_then(|s| s.as_str()),
            )
        })
    {
        (url.to_string(), version.to_string())
    } else {
        panic!(
            "Cargo.toml does not contain a valid [package.metadata.external-dependencies.{LIB_NAME}] table"
        );
    }
}

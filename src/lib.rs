//! [dav1d] AV1 デコーダーの Rust バインディング
//!
//! [dav1d]: https://github.com/videolan/dav1d
#![warn(missing_docs)]

use std::{ffi::CStr, ffi::c_int, mem::MaybeUninit};

mod sys;

/// リンクされている dav1d ライブラリのバージョン文字列を返す
///
/// # Panics
///
/// dav1d_version() が不正な UTF-8 を返した場合にパニックする。
/// dav1d のバージョン文字列は常に ASCII であるため、通常は発生しない
pub fn version() -> &'static str {
    unsafe {
        CStr::from_ptr(sys::dav1d_version())
            .to_str()
            .expect("dav1d_version() returned invalid UTF-8")
    }
}

/// リンクされている dav1d ライブラリの API バージョンを数値で返す
///
/// フォーマット: `((major << 16) | (minor << 8) | patch)`
pub fn version_api() -> u32 {
    unsafe { sys::dav1d_version_api() }
}

/// ビルド時に参照したリポジトリ URL
pub const BUILD_REPOSITORY: &str = sys::BUILD_METADATA_REPOSITORY;

/// ビルド時に参照したリポジトリのバージョン（タグ）
pub const BUILD_VERSION: &str = sys::BUILD_METADATA_VERSION;

/// エラー
#[derive(Debug)]
pub struct Error {
    code: c_int,
    function: &'static str,
}

impl Error {
    fn check(code: c_int, function: &'static str) -> Result<(), Self> {
        if code == 0 {
            Ok(())
        } else {
            Err(Self { code, function })
        }
    }

    /// エラーコードが EAGAIN かどうかを返す
    ///
    /// [`Decoder::decode()`] が EAGAIN を返した場合、
    /// 先に [`Decoder::next_frame()`] でフレームを取り出してから再度デコードする必要がある
    pub fn is_eagain(&self) -> bool {
        self.code.unsigned_abs() == sys::EAGAIN
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errno_name = match self.code.unsigned_abs() {
            sys::EAGAIN => " (EAGAIN)",
            sys::ENOMEM => " (ENOMEM)",
            _ => "",
        };
        write!(
            f,
            "{}() failed: code={}{}",
            self.function, self.code, errno_name
        )
    }
}

impl std::error::Error for Error {}

/// ピクセルレイアウト
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelLayout {
    /// モノクロ (Y のみ)
    I400,
    /// YUV 4:2:0
    I420,
    /// YUV 4:2:2
    I422,
    /// YUV 4:4:4
    I444,
}

/// 色域 (H.273 準拠)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPrimaries {
    /// ITU-R BT.709
    BT709,
    /// 不明
    Unknown,
    /// ITU-R BT.470M
    BT470M,
    /// ITU-R BT.470BG
    BT470BG,
    /// ITU-R BT.601
    BT601,
    /// SMPTE 240M
    SMPTE240,
    /// フィルム
    Film,
    /// ITU-R BT.2020
    BT2020,
    /// CIE XYZ
    XYZ,
    /// SMPTE 431 (DCI-P3)
    SMPTE431,
    /// SMPTE 432 (Display P3)
    SMPTE432,
    /// EBU 3213-E
    EBU3213,
    /// 予約済み / 未対応の値
    Reserved,
}

impl ColorPrimaries {
    fn from_raw(raw: sys::Dav1dColorPrimaries) -> Self {
        match raw {
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT709 => Self::BT709,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_UNKNOWN => Self::Unknown,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT470M => Self::BT470M,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT470BG => Self::BT470BG,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT601 => Self::BT601,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE240 => Self::SMPTE240,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_FILM => Self::Film,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_BT2020 => Self::BT2020,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_XYZ => Self::XYZ,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE431 => Self::SMPTE431,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_SMPTE432 => Self::SMPTE432,
            sys::Dav1dColorPrimaries_DAV1D_COLOR_PRI_EBU3213 => Self::EBU3213,
            _ => Self::Reserved,
        }
    }
}

/// 伝達特性 (H.273 準拠)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferCharacteristics {
    /// ITU-R BT.709
    BT709,
    /// 不明
    Unknown,
    /// ITU-R BT.470M
    BT470M,
    /// ITU-R BT.470BG
    BT470BG,
    /// ITU-R BT.601
    BT601,
    /// SMPTE 240M
    SMPTE240,
    /// リニア
    Linear,
    /// 対数 (100:1)
    Log100,
    /// 対数 (100*sqrt(10):1)
    Log100Sqrt10,
    /// IEC 61966-2-4
    IEC61966,
    /// ITU-R BT.1361
    BT1361,
    /// sRGB
    Srgb,
    /// ITU-R BT.2020 10-bit
    BT2020_10bit,
    /// ITU-R BT.2020 12-bit
    BT2020_12bit,
    /// SMPTE 2084 (PQ)
    SMPTE2084,
    /// SMPTE 428
    SMPTE428,
    /// HLG (Hybrid Log-Gamma)
    Hlg,
    /// 予約済み / 未対応の値
    Reserved,
}

impl TransferCharacteristics {
    fn from_raw(raw: sys::Dav1dTransferCharacteristics) -> Self {
        match raw {
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT709 => Self::BT709,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_UNKNOWN => Self::Unknown,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT470M => Self::BT470M,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT470BG => Self::BT470BG,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT601 => Self::BT601,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE240 => Self::SMPTE240,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_LINEAR => Self::Linear,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_LOG100 => Self::Log100,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_LOG100_SQRT10 => Self::Log100Sqrt10,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_IEC61966 => Self::IEC61966,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT1361 => Self::BT1361,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_SRGB => Self::Srgb,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT2020_10BIT => Self::BT2020_10bit,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_BT2020_12BIT => Self::BT2020_12bit,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE2084 => Self::SMPTE2084,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_SMPTE428 => Self::SMPTE428,
            sys::Dav1dTransferCharacteristics_DAV1D_TRC_HLG => Self::Hlg,
            _ => Self::Reserved,
        }
    }
}

/// 行列係数 (H.273 準拠)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixCoefficients {
    /// Identity (RGB)
    Identity,
    /// ITU-R BT.709
    BT709,
    /// 不明
    Unknown,
    /// FCC
    FCC,
    /// ITU-R BT.470BG
    BT470BG,
    /// ITU-R BT.601
    BT601,
    /// SMPTE 240M
    SMPTE240,
    /// SMPTE YCgCo
    SMPTEYCgCo,
    /// ITU-R BT.2020 非定数輝度
    BT2020NCL,
    /// ITU-R BT.2020 定数輝度
    BT2020CL,
    /// SMPTE 2085
    SMPTE2085,
    /// クロマティシティ導出非定数輝度
    ChromatNCL,
    /// クロマティシティ導出定数輝度
    ChromatCL,
    /// ICtCp
    ICtCp,
    /// 予約済み / 未対応の値
    Reserved,
}

impl MatrixCoefficients {
    fn from_raw(raw: sys::Dav1dMatrixCoefficients) -> Self {
        match raw {
            sys::Dav1dMatrixCoefficients_DAV1D_MC_IDENTITY => Self::Identity,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_BT709 => Self::BT709,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_UNKNOWN => Self::Unknown,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_FCC => Self::FCC,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_BT470BG => Self::BT470BG,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_BT601 => Self::BT601,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_SMPTE240 => Self::SMPTE240,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_SMPTE_YCGCO => Self::SMPTEYCgCo,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_BT2020_NCL => Self::BT2020NCL,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_BT2020_CL => Self::BT2020CL,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_SMPTE2085 => Self::SMPTE2085,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_CHROMAT_NCL => Self::ChromatNCL,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_CHROMAT_CL => Self::ChromatCL,
            sys::Dav1dMatrixCoefficients_DAV1D_MC_ICTCP => Self::ICtCp,
            _ => Self::Reserved,
        }
    }
}

/// クロマサンプル位置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChromaSamplePosition {
    /// 不明
    Unknown,
    /// 垂直方向 (MPEG-2 互換)
    Vertical,
    /// コロケーテッド (H.264/HEVC 互換)
    Colocated,
}

impl ChromaSamplePosition {
    fn from_raw(raw: sys::Dav1dChromaSamplePosition) -> Self {
        match raw {
            sys::Dav1dChromaSamplePosition_DAV1D_CHR_UNKNOWN => Self::Unknown,
            sys::Dav1dChromaSamplePosition_DAV1D_CHR_VERTICAL => Self::Vertical,
            sys::Dav1dChromaSamplePosition_DAV1D_CHR_COLOCATED => Self::Colocated,
            _ => Self::Unknown,
        }
    }
}

/// 色域レンジ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    /// スタジオレンジ (MPEG、8-bit: Y \[16,235\], C \[16,240\])
    Studio,
    /// フルレンジ (JPEG、8-bit: \[0,255\])
    Full,
}

impl ColorRange {
    fn from_raw(raw: u8) -> Self {
        match raw {
            1 => Self::Full,
            _ => Self::Studio,
        }
    }
}

/// シーケンスヘッダー情報
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceHeader {
    /// AV1 プロファイル (0, 1, 2)
    pub profile: u8,
    /// ストリームの最大幅
    pub max_width: usize,
    /// ストリームの最大高さ
    pub max_height: usize,
    /// ピクセルレイアウト
    pub layout: PixelLayout,
    /// 色域
    pub pri: ColorPrimaries,
    /// 伝達特性
    pub trc: TransferCharacteristics,
    /// 行列係数
    pub mtrx: MatrixCoefficients,
    /// クロマサンプル位置
    pub chr: ChromaSamplePosition,
    /// ビット深度 (8, 10, 12)
    pub bit_depth: usize,
    /// 色域レンジ
    pub color_range: ColorRange,
}

/// ビットストリームからシーケンスヘッダーを解析する
///
/// デコーダーを開かずにストリームのメタデータを事前に取得できる。
/// シーケンスヘッダー以外の OBU が含まれていても無視される
pub fn parse_sequence_header(data: &[u8]) -> Result<SequenceHeader, Error> {
    let mut seq_hdr = MaybeUninit::<sys::Dav1dSequenceHeader>::zeroed();
    unsafe {
        let code =
            sys::dav1d_parse_sequence_header(seq_hdr.as_mut_ptr(), data.as_ptr(), data.len());
        Error::check(code, "dav1d_parse_sequence_header")?;
        let seq_hdr = seq_hdr.assume_init();
        let layout = match seq_hdr.layout {
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I400 => PixelLayout::I400,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I420 => PixelLayout::I420,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I422 => PixelLayout::I422,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I444 => PixelLayout::I444,
            _ => unreachable!("unknown pixel layout: {}", seq_hdr.layout),
        };
        Ok(SequenceHeader {
            profile: seq_hdr.profile,
            max_width: seq_hdr.max_width as usize,
            max_height: seq_hdr.max_height as usize,
            layout,
            pri: ColorPrimaries::from_raw(seq_hdr.pri),
            trc: TransferCharacteristics::from_raw(seq_hdr.trc),
            mtrx: MatrixCoefficients::from_raw(seq_hdr.mtrx),
            chr: ChromaSamplePosition::from_raw(seq_hdr.chr),
            bit_depth: match seq_hdr.hbd {
                0 => 8,
                1 => 10,
                2 => 12,
                _ => unreachable!("unknown hbd value: {}", seq_hdr.hbd),
            },
            color_range: ColorRange::from_raw(seq_hdr.color_range),
        })
    }
}

/// デコードエラーに関連するデータプロパティ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataProps {
    /// コンテナのタイムスタンプ (不明の場合は `i64::MIN`)
    pub timestamp: i64,
    /// コンテナの再生時間 (不明の場合は 0)
    pub duration: i64,
    /// ストリーム内のオフセット (不明の場合は -1)
    pub offset: i64,
    /// パケットサイズ
    pub size: usize,
}

/// コンテンツライトレベル (HDR メタデータ)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentLightLevel {
    /// 最大コンテンツ輝度 (cd/m²)
    pub max_content_light_level: u16,
    /// 最大フレーム平均輝度 (cd/m²)
    pub max_frame_average_light_level: u16,
}

/// マスタリングディスプレイ情報 (HDR メタデータ)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MasteringDisplay {
    /// 色域の原色座標 (R, G, B)、各 (x, y) は 0.16 固定小数点
    pub primaries: [[u16; 2]; 3],
    /// ホワイトポイント座標 (x, y)、0.16 固定小数点
    pub white_point: [u16; 2],
    /// 最大輝度、24.8 固定小数点 (cd/m²)
    pub max_luminance: u32,
    /// 最小輝度、18.14 固定小数点 (cd/m²)
    pub min_luminance: u32,
}

/// フレーム種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    /// キーフレーム
    Key,
    /// インターフレーム
    Inter,
    /// イントラオンリーフレーム
    Intra,
    /// スイッチフレーム
    Switch,
}

impl FrameType {
    fn from_raw(raw: sys::Dav1dFrameType) -> Self {
        match raw {
            sys::Dav1dFrameType_DAV1D_FRAME_TYPE_KEY => Self::Key,
            sys::Dav1dFrameType_DAV1D_FRAME_TYPE_INTER => Self::Inter,
            sys::Dav1dFrameType_DAV1D_FRAME_TYPE_INTRA => Self::Intra,
            sys::Dav1dFrameType_DAV1D_FRAME_TYPE_SWITCH => Self::Switch,
            _ => unreachable!("unknown frame type: {raw}"),
        }
    }
}

/// インループフィルターの種別 (ビットフラグ)
///
/// dav1d ではビットフラグとして定義されており、
/// ビット OR で組み合わせて使用する
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InloopFilterType(sys::Dav1dInloopFilterType);

impl InloopFilterType {
    /// フィルターなし (0)
    pub const NONE: Self = Self(sys::Dav1dInloopFilterType_DAV1D_INLOOPFILTER_NONE);
    /// デブロッキングフィルター (1)
    pub const DEBLOCK: Self = Self(sys::Dav1dInloopFilterType_DAV1D_INLOOPFILTER_DEBLOCK);
    /// CDEF フィルター (2)
    pub const CDEF: Self = Self(sys::Dav1dInloopFilterType_DAV1D_INLOOPFILTER_CDEF);
    /// ループリストレーションフィルター (4)
    pub const RESTORATION: Self = Self(sys::Dav1dInloopFilterType_DAV1D_INLOOPFILTER_RESTORATION);
    /// 全フィルター有効 (7)
    pub const ALL: Self = Self(sys::Dav1dInloopFilterType_DAV1D_INLOOPFILTER_ALL);

    fn to_raw(self) -> sys::Dav1dInloopFilterType {
        self.0
    }
}

impl std::ops::BitOr for InloopFilterType {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// デコードイベントフラグ (ビットフラグ)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFlags(sys::Dav1dEventFlags);

impl EventFlags {
    /// 新しいシーケンスヘッダーが検出された
    pub const NEW_SEQUENCE: Self = Self(sys::Dav1dEventFlags_DAV1D_EVENT_FLAG_NEW_SEQUENCE);
    /// 現在のシーケンスで新しいオペレーティングパラメータが検出された
    pub const NEW_OP_PARAMS_INFO: Self =
        Self(sys::Dav1dEventFlags_DAV1D_EVENT_FLAG_NEW_OP_PARAMS_INFO);

    /// 指定したフラグが含まれているかを判定する
    pub fn contains(self, flag: Self) -> bool {
        self.0 & flag.0 == flag.0
    }
}

/// デコードするフレーム種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeFrameType {
    /// 全フレームをデコード
    All,
    /// 参照フレームのみ
    Reference,
    /// イントラフレームのみ
    Intra,
    /// キーフレームのみ
    Key,
}

impl DecodeFrameType {
    fn to_raw(self) -> sys::Dav1dDecodeFrameType {
        match self {
            Self::All => sys::Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_ALL,
            Self::Reference => sys::Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_REFERENCE,
            Self::Intra => sys::Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_INTRA,
            Self::Key => sys::Dav1dDecodeFrameType_DAV1D_DECODEFRAMETYPE_KEY,
        }
    }
}

/// デコーダーの設定
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// デコードに使用するスレッド数
    pub n_threads: usize,
    /// 最大フレーム遅延 (0 で dav1d が自動決定)
    pub max_frame_delay: usize,
    /// フィルムグレインを適用するかどうか
    pub apply_grain: bool,
    /// スケーラブル AV1 のオペレーティングポイント (0-31)
    pub operating_point: usize,
    /// スケーラブル AV1 の全空間レイヤーを出力するか
    pub all_layers: bool,
    /// 最大フレームサイズ制限 (ピクセル単位、None で無制限)
    pub frame_size_limit: Option<u32>,
    /// ビットストリーム規格違反時にデコードを中断するか
    pub strict_std_compliance: bool,
    /// 非表示フレームも出力するか
    pub output_invisible_frames: bool,
    /// 有効にするインループフィルター
    pub inloop_filters: InloopFilterType,
    /// デコードするフレーム種別
    pub decode_frame_type: DecodeFrameType,
}

impl DecoderConfig {
    /// デフォルト設定でインスタンスを生成する
    ///
    /// マルチスレッドを使用する場合は `n_threads` を変更すること
    pub fn new() -> Self {
        Self {
            n_threads: 1,
            max_frame_delay: 0,
            apply_grain: true,
            operating_point: 0,
            all_layers: true,
            frame_size_limit: None,
            strict_std_compliance: false,
            output_invisible_frames: false,
            inloop_filters: InloopFilterType::ALL,
            decode_frame_type: DecodeFrameType::All,
        }
    }

    /// この設定でのデコーダーのフレーム遅延を取得する
    ///
    /// 戻り値は 1 以上 `max_frame_delay` 以下であることが保証される
    pub fn frame_delay(&self) -> Result<usize, Error> {
        let mut settings = MaybeUninit::<sys::Dav1dSettings>::zeroed();
        unsafe {
            sys::dav1d_default_settings(settings.as_mut_ptr());
            let mut settings = settings.assume_init();
            settings.n_threads = self.n_threads as c_int;
            settings.max_frame_delay = self.max_frame_delay as c_int;
            let code = sys::dav1d_get_frame_delay(&settings);
            if code < 0 {
                return Err(Error {
                    code,
                    function: "dav1d_get_frame_delay",
                });
            }
            Ok(code as usize)
        }
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// AV1 デコーダー
#[derive(Debug)]
pub struct Decoder {
    ctx: *mut sys::Dav1dContext,
}

impl Decoder {
    /// AV1 デコーダーインスタンスを生成する
    pub fn new(config: DecoderConfig) -> Result<Self, Error> {
        let mut settings = MaybeUninit::<sys::Dav1dSettings>::zeroed();
        unsafe {
            sys::dav1d_default_settings(settings.as_mut_ptr());

            let mut settings = settings.assume_init();
            settings.n_threads = config.n_threads as c_int;
            settings.max_frame_delay = config.max_frame_delay as c_int;
            settings.apply_grain = config.apply_grain as c_int;
            settings.operating_point = config.operating_point as c_int;
            settings.all_layers = config.all_layers as c_int;
            settings.frame_size_limit = config.frame_size_limit.unwrap_or(0) as _;
            settings.strict_std_compliance = config.strict_std_compliance as c_int;
            settings.output_invisible_frames = config.output_invisible_frames as c_int;
            settings.inloop_filters = config.inloop_filters.to_raw();
            settings.decode_frame_type = config.decode_frame_type.to_raw();

            let mut ctx = std::ptr::null_mut();
            let code = sys::dav1d_open(&mut ctx, &settings);
            Error::check(code, "dav1d_open")?;

            Ok(Self { ctx })
        }
    }

    /// 圧縮された映像フレームをデコードする
    ///
    /// デコード結果は [`Decoder::next_frame()`] で取得できる。
    /// データは dav1d の内部バッファにコピーされる。
    ///
    /// dav1d の内部バッファが満杯の場合はエラー (EAGAIN) を返す。
    /// その場合は先に [`Decoder::next_frame()`] でフレームを取り出してから再度呼び出すこと
    pub fn decode(&mut self, data: &[u8]) -> Result<(), Error> {
        let mut dav1d_data = MaybeUninit::<sys::Dav1dData>::zeroed();
        unsafe {
            let dav1d_data_buf_ptr = sys::dav1d_data_create(dav1d_data.as_mut_ptr(), data.len());
            if dav1d_data_buf_ptr.is_null() {
                // dav1d の慣習に倣ってエラーコードは負数にする
                Error::check(-(sys::ENOMEM as c_int), "dav1d_data_create")?;
            }
            std::slice::from_raw_parts_mut(dav1d_data_buf_ptr, data.len()).copy_from_slice(data);

            let mut dav1d_data = dav1d_data.assume_init();
            let code = sys::dav1d_send_data(self.ctx, &mut dav1d_data);
            Error::check(code, "dav1d_send_data").inspect_err(|_| {
                sys::dav1d_data_unref(&mut dav1d_data);
            })?;
        }
        Ok(())
    }

    /// これ以上データが来ないことをデコーダーに伝える
    ///
    /// 残りのデコード結果は [`Decoder::next_frame()`] で取得できる
    pub fn finish(&mut self) -> Result<(), Error> {
        // [NOTE]
        // dav1d では dav1d_get_picture() が EAGAIN を返した後にもう一度
        // 同じ関数を呼び出すと、強制的にバッファ内のデコード画像取得されるようになる。
        // そのため、finish() の中で特にやることはないが、他のライブラリのデコーダのインタフェースに
        // 合わせておいた方が分かりやすいので、メソッドの枠だけは用意している。
        Ok(())
    }

    /// デコード中に発生したイベントフラグを取得する
    ///
    /// 呼び出すと内部のフラグはクリアされる
    pub fn get_event_flags(&mut self) -> Result<EventFlags, Error> {
        let mut flags: sys::Dav1dEventFlags = 0;
        unsafe {
            let code = sys::dav1d_get_event_flags(self.ctx, &mut flags);
            Error::check(code, "dav1d_get_event_flags")?;
        }
        Ok(EventFlags(flags))
    }

    /// 最後のデコードエラーに関連するデータプロパティを取得する
    pub fn get_decode_error_data_props(&mut self) -> Result<DataProps, Error> {
        let mut props = MaybeUninit::<sys::Dav1dDataProps>::zeroed();
        unsafe {
            let code = sys::dav1d_get_decode_error_data_props(self.ctx, props.as_mut_ptr());
            Error::check(code, "dav1d_get_decode_error_data_props")?;
            let props = props.assume_init();
            Ok(DataProps {
                timestamp: props.timestamp,
                duration: props.duration,
                offset: props.offset,
                size: props.size,
            })
        }
    }

    /// デコード済みフレームにフィルムグレインを適用する
    ///
    /// `DecoderConfig::apply_grain` が `false` の場合に、
    /// 選択したフレームだけに後からグレインを適用できる。
    /// フレームにグレインメタデータがない場合は新しい参照を返す
    pub fn apply_grain(&mut self, frame: &DecodedFrame) -> Result<DecodedFrame, Error> {
        let mut out = MaybeUninit::<sys::Dav1dPicture>::zeroed();
        unsafe {
            let code = sys::dav1d_apply_grain(self.ctx, out.as_mut_ptr(), &frame.0);
            Error::check(code, "dav1d_apply_grain")?;
            Ok(DecodedFrame(out.assume_init()))
        }
    }

    /// デコーダーの内部状態をリセットする
    ///
    /// ストリーム内でシークした後に呼び出すことで、
    /// デコーダーを新しい位置からのデコードに備えさせる。
    /// 未消費のデータやバッファ中のフレームは全て破棄される
    pub fn flush(&mut self) {
        unsafe {
            sys::dav1d_flush(self.ctx);
        }
    }

    /// デコード済みのフレームを取り出す
    ///
    /// [`Decoder::decode()`] や [`Decoder::finish()`] の後には、
    /// このメソッドを、結果が `None` になるまで呼び出し続ける必要がある
    pub fn next_frame(&mut self) -> Result<Option<DecodedFrame>, Error> {
        let mut picture = MaybeUninit::<sys::Dav1dPicture>::zeroed();
        unsafe {
            let code = sys::dav1d_get_picture(self.ctx, picture.as_mut_ptr());
            if code < 0 && code.unsigned_abs() == sys::EAGAIN {
                return Ok(None);
            }
            Error::check(code, "dav1d_get_picture")?;

            let picture = picture.assume_init();
            Ok(Some(DecodedFrame(picture)))
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            sys::dav1d_close(&mut self.ctx);
        }
    }
}

// SAFETY: Dav1dContext はスレッドローカルな状態を持たず、全ての操作
// (dav1d_open, dav1d_send_data, dav1d_get_picture, dav1d_flush, dav1d_close 等) は
// コンテキストポインタ経由でのみ状態にアクセスする。そのため、所有権の移動
// (ある時点で一つのスレッドのみがアクセスする) は安全である。
// &mut self メソッドのみ提供するため Sync は不要
// (&Decoder を複数スレッドで共有しても呼び出せるメソッドがない)
unsafe impl Send for Decoder {}

/// デコードされた映像フレーム
#[derive(Debug)]
pub struct DecodedFrame(sys::Dav1dPicture);

impl DecodedFrame {
    /// フレームのピクセルレイアウトを返す
    pub fn pixel_layout(&self) -> PixelLayout {
        match self.0.p.layout {
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I400 => PixelLayout::I400,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I420 => PixelLayout::I420,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I422 => PixelLayout::I422,
            sys::Dav1dPixelLayout_DAV1D_PIXEL_LAYOUT_I444 => PixelLayout::I444,
            _ => unreachable!("unknown pixel layout: {}", self.0.p.layout),
        }
    }

    /// フレームのビット深度を返す (8, 10, 12)
    pub fn bit_depth(&self) -> usize {
        self.0.p.bpc as usize
    }

    /// ハイビット深度 (10-bit 以上) かどうかを返す
    pub fn is_high_depth(&self) -> bool {
        self.bit_depth() > 8
    }

    /// フレームの Y 成分のデータを返す
    ///
    /// ハイビット深度の場合は 1 ピクセルあたり 2 バイト (リトルエンディアン) になる。
    /// ストライドはバイト単位なのでハイビット深度でも既に考慮済み。
    /// 返されるスライスの長さは `height * stride` バイトで、各行末のパディングを含む
    pub fn y_plane(&self) -> &[u8] {
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する。
        // aligned_height >= height なので height * stride は常に有効な範囲内
        unsafe {
            std::slice::from_raw_parts(
                self.0.data[0].cast_const().cast(),
                self.height() * self.y_stride(),
            )
        }
    }

    /// フレームの U 成分のデータを返す
    ///
    /// ハイビット深度の場合は 1 ピクセルあたり 2 バイト (リトルエンディアン) になる。
    /// ピクセルレイアウトが [`PixelLayout::I400`] の場合は空のスライスを返す。
    /// 返されるスライスの長さは `chroma_height * stride` バイトで、各行末のパディングを含む
    pub fn u_plane(&self) -> &[u8] {
        if self.pixel_layout() == PixelLayout::I400 {
            return &[];
        }
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する
        unsafe {
            std::slice::from_raw_parts(
                self.0.data[1].cast_const().cast(),
                self.chroma_height() * self.u_stride(),
            )
        }
    }

    /// フレームの V 成分のデータを返す
    ///
    /// ハイビット深度の場合は 1 ピクセルあたり 2 バイト (リトルエンディアン) になる。
    /// ピクセルレイアウトが [`PixelLayout::I400`] の場合は空のスライスを返す。
    /// 返されるスライスの長さは `chroma_height * stride` バイトで、各行末のパディングを含む
    pub fn v_plane(&self) -> &[u8] {
        if self.pixel_layout() == PixelLayout::I400 {
            return &[];
        }
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する
        unsafe {
            std::slice::from_raw_parts(
                self.0.data[2].cast_const().cast(),
                self.chroma_height() * self.v_stride(),
            )
        }
    }

    /// フレームの Y 成分を u16 スライスとして返す
    ///
    /// ビット深度が 8 の場合は `None` を返す。
    /// 返されるスライスの長さは `height * (stride / 2)` 要素で、各行末のパディングを含む
    pub fn y_plane_u16(&self) -> Option<&[u16]> {
        if !self.is_high_depth() {
            return None;
        }
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する
        unsafe {
            Some(std::slice::from_raw_parts(
                self.0.data[0].cast_const().cast(),
                self.height() * self.y_stride() / 2,
            ))
        }
    }

    /// フレームの U 成分を u16 スライスとして返す
    ///
    /// ビット深度が 8 の場合、またはピクセルレイアウトが [`PixelLayout::I400`] の場合は `None` を返す。
    /// 返されるスライスの長さは `chroma_height * (stride / 2)` 要素で、各行末のパディングを含む
    pub fn u_plane_u16(&self) -> Option<&[u16]> {
        if !self.is_high_depth() || self.pixel_layout() == PixelLayout::I400 {
            return None;
        }
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する
        unsafe {
            Some(std::slice::from_raw_parts(
                self.0.data[1].cast_const().cast(),
                self.chroma_height() * self.u_stride() / 2,
            ))
        }
    }

    /// フレームの V 成分を u16 スライスとして返す
    ///
    /// ビット深度が 8 の場合、またはピクセルレイアウトが [`PixelLayout::I400`] の場合は `None` を返す。
    /// 返されるスライスの長さは `chroma_height * (stride / 2)` 要素で、各行末のパディングを含む
    pub fn v_plane_u16(&self) -> Option<&[u16]> {
        if !self.is_high_depth() || self.pixel_layout() == PixelLayout::I400 {
            return None;
        }
        // SAFETY: dav1d のデフォルトアロケータは stride * aligned_height バイトを確保する
        unsafe {
            Some(std::slice::from_raw_parts(
                self.0.data[2].cast_const().cast(),
                self.chroma_height() * self.v_stride() / 2,
            ))
        }
    }

    /// フレームの Y 成分のストライドを返す (バイト単位)
    ///
    /// dav1d の stride は `ptrdiff_t` 型だが、負の stride (ボトムアップ画像) はサポートしない
    pub fn y_stride(&self) -> usize {
        assert!(self.0.stride[0] >= 0, "negative stride is not supported");
        self.0.stride[0] as usize
    }

    /// フレームの U 成分のストライドを返す (バイト単位)
    ///
    /// dav1d の stride は `ptrdiff_t` 型だが、負の stride (ボトムアップ画像) はサポートしない。
    /// [`PixelLayout::I400`] の場合でも dav1d の内部値が返されるが、
    /// クロマプレーン自体は存在しないため [`u_plane()`](Self::u_plane) は空のスライスを返す
    pub fn u_stride(&self) -> usize {
        assert!(self.0.stride[1] >= 0, "negative stride is not supported");
        self.0.stride[1] as usize
    }

    /// フレームの V 成分のストライドを返す (バイト単位)
    ///
    /// [`PixelLayout::I400`] の場合でも dav1d の内部値が返されるが、
    /// クロマプレーン自体は存在しないため [`v_plane()`](Self::v_plane) は空のスライスを返す
    pub fn v_stride(&self) -> usize {
        self.u_stride() // U と V は共通
    }

    /// フレームの幅を返す
    pub fn width(&self) -> usize {
        self.0.p.w as usize
    }

    /// フレームの高さを返す
    pub fn height(&self) -> usize {
        self.0.p.h as usize
    }

    /// フレーム種別を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のフレームヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn frame_type(&self) -> FrameType {
        assert!(!self.0.frame_hdr.is_null(), "frame_hdr is null");
        unsafe { FrameType::from_raw((*self.0.frame_hdr).frame_type) }
    }

    /// SVC 用テンポラル ID を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のフレームヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn temporal_id(&self) -> u8 {
        assert!(!self.0.frame_hdr.is_null(), "frame_hdr is null");
        unsafe { (*self.0.frame_hdr).temporal_id }
    }

    /// SVC 用スパーシャル ID を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のフレームヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn spatial_id(&self) -> u8 {
        assert!(!self.0.frame_hdr.is_null(), "frame_hdr is null");
        unsafe { (*self.0.frame_hdr).spatial_id }
    }

    /// 表示フレームかどうかを返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のフレームヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn show_frame(&self) -> bool {
        assert!(!self.0.frame_hdr.is_null(), "frame_hdr is null");
        unsafe { (*self.0.frame_hdr).show_frame != 0 }
    }

    /// 色域を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn color_primaries(&self) -> ColorPrimaries {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { ColorPrimaries::from_raw((*self.0.seq_hdr).pri) }
    }

    /// 伝達特性を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn transfer_characteristics(&self) -> TransferCharacteristics {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { TransferCharacteristics::from_raw((*self.0.seq_hdr).trc) }
    }

    /// 行列係数を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn matrix_coefficients(&self) -> MatrixCoefficients {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { MatrixCoefficients::from_raw((*self.0.seq_hdr).mtrx) }
    }

    /// クロマサンプル位置を返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn chroma_sample_position(&self) -> ChromaSamplePosition {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { ChromaSamplePosition::from_raw((*self.0.seq_hdr).chr) }
    }

    /// 色域レンジを返す
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn color_range(&self) -> ColorRange {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { ColorRange::from_raw((*self.0.seq_hdr).color_range) }
    }

    /// AV1 プロファイルを返す (0, 1, 2)
    ///
    /// # Panics
    ///
    /// dav1d 内部のシーケンスヘッダーポインタが null の場合にパニックする。
    /// [`Decoder::next_frame()`] から取得したフレームでは発生しない
    pub fn profile(&self) -> u8 {
        assert!(!self.0.seq_hdr.is_null(), "seq_hdr is null");
        unsafe { (*self.0.seq_hdr).profile }
    }

    /// コンテンツライトレベルを返す (HDR メタデータ)
    ///
    /// メタデータが存在しない場合は `None` を返す
    pub fn content_light_level(&self) -> Option<ContentLightLevel> {
        if self.0.content_light.is_null() {
            return None;
        }
        unsafe {
            let cll = &*self.0.content_light;
            Some(ContentLightLevel {
                max_content_light_level: cll.max_content_light_level,
                max_frame_average_light_level: cll.max_frame_average_light_level,
            })
        }
    }

    /// マスタリングディスプレイ情報を返す (HDR メタデータ)
    ///
    /// メタデータが存在しない場合は `None` を返す
    pub fn mastering_display(&self) -> Option<MasteringDisplay> {
        if self.0.mastering_display.is_null() {
            return None;
        }
        unsafe {
            let md = &*self.0.mastering_display;
            Some(MasteringDisplay {
                primaries: md.primaries,
                white_point: md.white_point,
                max_luminance: md.max_luminance,
                min_luminance: md.min_luminance,
            })
        }
    }

    /// クロマ成分の高さを返す
    fn chroma_height(&self) -> usize {
        match self.pixel_layout() {
            PixelLayout::I400 => 0,
            PixelLayout::I420 => self.height().div_ceil(2),
            PixelLayout::I422 | PixelLayout::I444 => self.height(),
        }
    }
}

impl Drop for DecodedFrame {
    fn drop(&mut self) {
        unsafe {
            sys::dav1d_picture_unref(&mut self.0);
        }
    }
}

// SAFETY: Dav1dPicture の所有権移動が安全である理由:
// - dav1d_picture_unref() は Dav1dPicture 内のリファレンスカウントをアトミック操作
//   (atomic_fetch_sub) でデクリメントするため、別スレッドでの drop が安全
// - DecodedFrame は &mut self メソッドを持たず、全アクセサが &self で読み取り専用のため、
//   所有権移動後のアクセスパターンに問題はない
// - Sync も安全: 全アクセサが &self で読み取り専用かつ内部状態を変更しないため、
//   &DecodedFrame を複数スレッドから同時に参照しても競合は発生しない。
//   Arc<DecodedFrame> での共有も安全
unsafe impl Send for DecodedFrame {}
unsafe impl Sync for DecodedFrame {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_seq_hdr() {
        let data = [
            10, 11, 0, 0, 0, 36, 196, 255, 223, 63, 254, 96, 16, 50, 35, 16, 0, 144, 0, 0, 0, 160,
            0, 0, 128, 1, 197, 120, 80, 103, 179, 239, 241, 100, 76, 173, 116, 93, 183, 31, 101,
            221, 87, 90, 233, 219, 28, 199, 243, 128,
        ];
        let hdr = parse_sequence_header(&data).expect("parse error");
        assert_eq!(hdr.profile, 0);
        assert_eq!(hdr.layout, PixelLayout::I420);
    }

    #[test]
    fn init_decoder() {
        let config = DecoderConfig::new();
        assert!(Decoder::new(config).is_ok());
    }

    #[test]
    fn init_decoder_default() {
        let config = DecoderConfig::default();
        assert!(Decoder::new(config).is_ok());
    }

    #[test]
    fn decode_black() {
        let data = [
            10, 11, 0, 0, 0, 36, 196, 255, 223, 63, 254, 96, 16, 50, 35, 16, 0, 144, 0, 0, 0, 160,
            0, 0, 128, 1, 197, 120, 80, 103, 179, 239, 241, 100, 76, 173, 116, 93, 183, 31, 101,
            221, 87, 90, 233, 219, 28, 199, 243, 128,
        ];
        let config = DecoderConfig::new();
        let mut decoder = Decoder::new(config).expect("new() error");
        let mut count = 0;

        decoder.decode(&data).expect("decode() error");
        while let Ok(Some(frame)) = decoder.next_frame() {
            assert_eq!(frame.pixel_layout(), PixelLayout::I420);
            assert_eq!(frame.bit_depth(), 8);
            assert!(!frame.is_high_depth());
            assert!(!frame.y_plane().is_empty());
            assert!(!frame.u_plane().is_empty());
            assert!(!frame.v_plane().is_empty());
            assert_eq!(frame.frame_type(), FrameType::Key);
            assert!(frame.show_frame());
            assert_eq!(frame.profile(), 0);
            count += 1;
        }

        decoder.finish().expect("finish() error");
        while let Ok(Some(_)) = decoder.next_frame() {
            count += 1;
        }

        assert_eq!(count, 1);
    }
}

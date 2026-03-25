//! コーデック情報の照会

/// コーデック種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodecType {
    /// AV1
    Av1,
}

impl VideoCodecType {
    /// すべてのコーデック種別を返す
    fn all() -> &'static [Self] {
        &[Self::Av1]
    }
}

/// コーデックごとの情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecInfo {
    /// コーデック種別
    pub codec: VideoCodecType,
    /// デコード情報
    pub decoding: DecodingInfo,
    /// エンコード情報
    pub encoding: EncodingInfo,
}

/// デコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodingInfo {
    /// デコードが可能か
    pub supported: bool,
    /// ハードウェアアクセラレーションが利用可能か
    pub hardware_accelerated: bool,
    /// コーデック固有のプロファイル情報
    pub profiles: DecodingProfiles,
}

/// エンコード情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingInfo {
    /// エンコードが可能か
    pub supported: bool,
    /// ハードウェアアクセラレーションが利用可能か
    pub hardware_accelerated: bool,
    /// コーデック固有のプロファイル情報
    pub profiles: EncodingProfiles,
}

/// コーデック固有のデコードプロファイル情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingProfiles {
    /// AV1 プロファイル一覧
    Av1(Vec<Av1DecodingProfile>),
}

/// コーデック固有のエンコードプロファイル情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodingProfiles {
    /// プロファイル情報なし（エンコード非対応）
    None,
}

/// AV1 デコードプロファイル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Av1DecodingProfile {
    /// Main (8-bit/10-bit 4:2:0)
    Main,
    /// High (8-bit/10-bit 4:2:0/4:4:4)
    High,
    /// Professional (8-bit/10-bit/12-bit 全サブサンプリング)
    Professional,
}

/// このバックエンドで利用可能なコーデック情報の一覧を返す
///
/// dav1d はソフトウェアデコーダー専用であるため、AV1 のデコードのみ可能で、
/// エンコードとハードウェアアクセラレーションは利用できない。
pub fn supported_codecs() -> Vec<CodecInfo> {
    VideoCodecType::all()
        .iter()
        .map(|&codec| CodecInfo {
            codec,
            decoding: decoding_info(),
            encoding: encoding_info(),
        })
        .collect()
}

/// デコード情報を返す
///
/// dav1d はソフトウェアデコーダーであるため、supported は常に true、
/// hardware_accelerated は常に false になる。
/// AV1 の全プロファイル (Main, High, Professional) をデコード可能。
fn decoding_info() -> DecodingInfo {
    DecodingInfo {
        supported: true,
        hardware_accelerated: false,
        profiles: DecodingProfiles::Av1(vec![
            Av1DecodingProfile::Main,
            Av1DecodingProfile::High,
            Av1DecodingProfile::Professional,
        ]),
    }
}

/// エンコード情報を返す
///
/// dav1d はデコーダー専用であるため、エンコードは常に非対応。
fn encoding_info() -> EncodingInfo {
    EncodingInfo {
        supported: false,
        hardware_accelerated: false,
        profiles: EncodingProfiles::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_codecs_returns_one_codec() {
        let codecs = supported_codecs();
        assert_eq!(codecs.len(), 1);
        assert_eq!(codecs[0].codec, VideoCodecType::Av1);
    }

    #[test]
    fn av1_codec_info() {
        let codecs = supported_codecs();
        let av1 = &codecs[0];
        assert_eq!(
            *av1,
            CodecInfo {
                codec: VideoCodecType::Av1,
                decoding: DecodingInfo {
                    supported: true,
                    hardware_accelerated: false,
                    profiles: DecodingProfiles::Av1(vec![
                        Av1DecodingProfile::Main,
                        Av1DecodingProfile::High,
                        Av1DecodingProfile::Professional,
                    ]),
                },
                encoding: EncodingInfo {
                    supported: false,
                    hardware_accelerated: false,
                    profiles: EncodingProfiles::None,
                },
            }
        );
    }
}

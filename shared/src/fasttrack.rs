use time::OffsetDateTime;

pub struct FastTrackRtcData {
    pub now: Option<OffsetDateTime>,
    pub alarm_status: bool,
}

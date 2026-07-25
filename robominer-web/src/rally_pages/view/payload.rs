#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RallyResultPayloadKind {
    VersionedJson,
    LegacyExecutable,
    Unsupported,
}

fn is_legacy_javascript_result_data(result_data: &str) -> bool {
    let trimmed = result_data.trim_start();
    trimmed.starts_with("var myRobots")
        || trimmed.starts_with("var myGround")
        || trimmed.starts_with("var myOreTypes")
}

pub(super) fn classify_rally_result_payload(result_data: &str) -> RallyResultPayloadKind {
    if is_legacy_javascript_result_data(result_data) {
        return RallyResultPayloadKind::LegacyExecutable;
    }

    match serde_json::from_str::<serde_json::Value>(result_data) {
        Ok(value) if is_valid_versioned_rally_payload(&value) => {
            RallyResultPayloadKind::VersionedJson
        }
        _ => RallyResultPayloadKind::Unsupported,
    }
}

fn is_valid_versioned_rally_payload(value: &serde_json::Value) -> bool {
    let version = value.get("v").and_then(|version| version.as_u64());
    if version != Some(1) && version != Some(2) {
        return false;
    }

    let Some(robots) = value
        .get("robots")
        .and_then(|robots| robots.get("robot"))
        .and_then(|robots| robots.as_array())
    else {
        return false;
    };

    for robot in robots {
        if !robot
            .get("locations")
            .and_then(|locations| locations.as_array())
            .is_some()
        {
            return false;
        }
    }

    let Some(ground) = value.get("ground") else {
        return false;
    };
    ground.get("sizeX").and_then(|size| size.as_i64()).is_some()
        && ground.get("sizeY").and_then(|size| size.as_i64()).is_some()
        && ground
            .get("positions")
            .and_then(|positions| positions.as_array())
            .is_some()
}

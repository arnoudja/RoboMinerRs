//! Part `<select>` markup for the robot config panel.

use std::collections::HashMap;

use crate::html::escape_html;

#[derive(Clone, Copy)]
pub(super) enum PartCapacityLabel {
    None,
    Ore,
    Battery,
}

pub(super) struct RobotPartSelect<'a> {
    pub(super) label: &'a str,
    pub(super) field_prefix: &'a str,
    pub(super) robot_id: i64,
    pub(super) type_id: i64,
    pub(super) current_part_id: i64,
    pub(super) current_part_name: &'a str,
    pub(super) part_asset_map:
        &'a HashMap<i64, Vec<&'a robominer_db::RobotConfigPartAssetStateRecord>>,
    pub(super) memory_control: bool,
    pub(super) capacity_label: PartCapacityLabel,
    pub(super) disabled_attr: &'a str,
    pub(super) current_memory_capacity: Option<i32>,
    pub(super) current_capacity: Option<i32>,
}

pub(super) fn render_robot_part_select(body: &mut String, select: RobotPartSelect<'_>) {
    let RobotPartSelect {
        label,
        field_prefix,
        robot_id,
        type_id,
        current_part_id,
        current_part_name,
        part_asset_map,
        memory_control,
        capacity_label,
        disabled_attr,
        current_memory_capacity,
        current_capacity,
    } = select;
    let id_attr = if memory_control {
        format!(r#" id="{field_prefix}{robot_id}""#)
    } else {
        String::new()
    };
    let current_capacity_attr = current_memory_capacity
        .map(|capacity| format!(r#" data-memory-capacity="{capacity}""#))
        .unwrap_or_default();
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">{}</span><select{id_attr} name="{}{}" class="tableitem robot-select"{disabled_attr}><option value="{}"{current_capacity_attr} selected="selected">{}</option>"#,
        label,
        field_prefix,
        robot_id,
        current_part_id,
        escape_html(&part_option_label(
            current_part_name,
            capacity_label,
            current_capacity.unwrap_or(0),
        ))
    ));
    for asset in part_asset_map.get(&type_id).into_iter().flatten() {
        if asset.unassigned > 0 && asset.robot_part_id != current_part_id {
            let capacity_attr = if asset.memory_capacity > 0 {
                format!(r#" data-memory-capacity="{}""#, asset.memory_capacity)
            } else {
                String::new()
            };
            body.push_str(&format!(
                r#"<option value="{}"{capacity_attr}>{}</option>"#,
                asset.robot_part_id,
                escape_html(&part_option_label(
                    &asset.part_name,
                    capacity_label,
                    asset_capacity(capacity_label, asset),
                ))
            ));
        }
    }
    body.push_str("</select></label>");
}

fn asset_capacity(
    capacity_label: PartCapacityLabel,
    asset: &robominer_db::RobotConfigPartAssetStateRecord,
) -> i32 {
    match capacity_label {
        PartCapacityLabel::None => 0,
        PartCapacityLabel::Ore => asset.ore_capacity,
        PartCapacityLabel::Battery => asset.battery_capacity,
    }
}

fn part_option_label(part_name: &str, capacity_label: PartCapacityLabel, capacity: i32) -> String {
    if capacity <= 0 {
        return part_name.to_string();
    }
    match capacity_label {
        PartCapacityLabel::None => part_name.to_string(),
        PartCapacityLabel::Ore => format!("{part_name} ({capacity} Ore)"),
        PartCapacityLabel::Battery => format!("{part_name} ({capacity} pc)"),
    }
}

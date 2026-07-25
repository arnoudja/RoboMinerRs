//! Part `<select>` markup for the robot config panel.

use std::collections::HashMap;

use crate::html::escape_html;

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
    pub(super) disabled_attr: &'a str,
    pub(super) current_memory_capacity: Option<i32>,
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
        disabled_attr,
        current_memory_capacity,
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
        escape_html(current_part_name)
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
                escape_html(&asset.part_name)
            ));
        }
    }
    body.push_str("</select></label>");
}

use std::collections::HashMap;

use crate::html::{escape_html, format_period, layout, selected_attr};
use crate::robot_page::RobotPageState;
use super::{
    add_robot_stat_entry, push_robot_highlight, render_robot_part_select,
    robot_apply_block_reason, robot_memory_percent,
};

pub(super) fn render_robot_page(
    username: String,
    hud: Option<&str>,
    state: &RobotPageState,
) -> String {
    let mut part_asset_map: HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>> =
        HashMap::new();
    for asset in &state.part_assets {
        part_asset_map.entry(asset.type_id).or_default().push(asset);
    }

    let pending_count = state.robots.iter().filter(|robot| robot.change_pending).count();

    let mut robots = state.robots.clone();
    robots.sort_by(|left, right| {
        right
            .change_pending
            .cmp(&left.change_pending)
            .then_with(|| left.robot_name.cmp(&right.robot_name))
    });

    let mut body = String::from(r#"<div class="robot-page">"#);
    render_robot_summary(&mut body, state.robots.len(), pending_count);
    render_robot_claim_banner(&mut body, state);
    render_robot_message(&mut body, state);

    if state.robots.is_empty() {
        body.push_str(
            r#"<p class="robot-empty">No robots yet. <a href="shop">Visit the shop</a> to buy parts and build your first robot.</p>"#,
        );
    } else {
        body.push_str(r#"<div class="robot-deck">"#);
        body.push_str(r#"<section class="robot-fleet" aria-labelledby="robot-fleet-title">"#);
        body.push_str(r#"<h2 id="robot-fleet-title" class="robot-section-title">Fleet</h2>"#);
        body.push_str(
            r#"<p class="robot-fleet-hint">Select a robot to configure parts and program source.</p>"#,
        );
        body.push_str(r#"<div class="robot-fleet-cards">"#);
        for robot in &robots {
            render_robot_fleet_card(
                &mut body,
                robot,
                &state.program_sources,
                robot.robot_id == state.selected_robot_id,
            );
        }
        body.push_str("</div></section>");
        body.push_str(r#"<div class="robot-config-area">"#);
        body.push_str(r#"<form id="robotForm" action="robot" method="post" class="robot-config-form">"#);
        for robot in &robots {
            render_robot_config_panel(
                &mut body,
                robot,
                robot.robot_id == state.selected_robot_id,
                &state.program_sources,
                &part_asset_map,
            );
        }
        body.push_str("</form></div></div>");
    }

    body.push_str(
        r#"<script>
(function() {
    var allowPageUnload = false;

    function syncRobotUrl(robotId) {
        if (window.history && window.history.replaceState) {
            window.history.replaceState(null, '', 'robot?robotId=' + encodeURIComponent(robotId));
        }
    }

    function setPanelEnabled(panel, enabled) {
        var fields = panel.querySelectorAll('input, select, button');
        for (var index = 0; index < fields.length; index += 1) {
            fields[index].disabled = !enabled;
        }
    }

    function panelFormSnapshot(panel) {
        var snapshot = {};
        var fields = panel.querySelectorAll('input[name], select[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && field.name !== 'robotId') {
                snapshot[field.name] = field.value;
            }
        }
        return JSON.stringify(snapshot);
    }

    function isPanelDirty(panel) {
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return false;
        }
        return panelFormSnapshot(panel) !== baseline;
    }

    function capturePanelBaseline(panel) {
        panel.setAttribute('data-form-baseline', panelFormSnapshot(panel));
    }

    function restorePanelBaseline(panel) {
        var baseline = panel.getAttribute('data-form-baseline');
        if (!baseline) {
            return;
        }
        var snapshot = JSON.parse(baseline);
        var fields = panel.querySelectorAll('input[name], select[name]');
        for (var index = 0; index < fields.length; index += 1) {
            var field = fields[index];
            if (field.name && field.name !== 'robotId' && Object.prototype.hasOwnProperty.call(snapshot, field.name)) {
                field.value = snapshot[field.name];
            }
        }
    }

    function updateRobotQuickLinks(panel) {
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var editLink = panel.querySelector('.robot-quick-link-edit-program');
        if (programSelect && editLink) {
            editLink.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programSelect.value);
        }
    }

    function updateRobotDirtyState(panel) {
        if (!panel) {
            return;
        }
        var dirty = isPanelDirty(panel);
        var readyBadge = panel.querySelector('.robot-status-ready');
        var dirtyBadge = panel.querySelector('.robot-status-dirty');
        var resetButton = panel.querySelector('.robot-reset-btn');
        if (readyBadge) {
            readyBadge.hidden = dirty;
        }
        if (dirtyBadge) {
            dirtyBadge.hidden = !dirty;
        }
        if (resetButton) {
            resetButton.hidden = !dirty;
        }
    }

    function selectRobot(robotId, updateUrl) {
        if (updateUrl === undefined) {
            updateUrl = true;
        }
        var cards = document.querySelectorAll('.robot-fleet-card');
        var panels = document.querySelectorAll('.robot-config-panel');
        for (var cardIndex = 0; cardIndex < cards.length; cardIndex += 1) {
            var card = cards[cardIndex];
            if (card.getAttribute('data-robot-id') === robotId) {
                card.classList.add('robot-fleet-card-active');
            } else {
                card.classList.remove('robot-fleet-card-active');
            }
        }
        for (var index = 0; index < panels.length; index += 1) {
            var panel = panels[index];
            var isActive = panel.getAttribute('data-robot-id') === robotId;
            panel.classList.toggle('robot-config-panel-active', isActive);
            panel.hidden = !isActive;
            setPanelEnabled(panel, isActive);
            if (isActive) {
                if (!panel.getAttribute('data-form-baseline')) {
                    capturePanelBaseline(panel);
                }
                updateRobotApplyState(panel);
            }
        }
        if (updateUrl) {
            syncRobotUrl(robotId);
        }
    }

    function updateRobotMemoryPreview(panel) {
        if (!panel) {
            return;
        }
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (!programSelect || !memorySelect) {
            return;
        }
        var programOption = programSelect.options[programSelect.selectedIndex];
        var memoryOption = memorySelect.options[memorySelect.selectedIndex];
        var programSize = parseInt(programOption.getAttribute('data-compiled-size') || '0', 10);
        var memorySize = parseInt(memoryOption.getAttribute('data-memory-capacity') || '0', 10);
        if (memorySize <= 0) {
            memorySize = 1;
        }
        var percent = Math.min(100, Math.max(0, (programSize / memorySize) * 100));
        var valueElement = panel.querySelector('.robot-progress-value');
        var barElement = panel.querySelector('.robot-progress-bar');
        if (valueElement) {
            valueElement.textContent = programSize + '/' + memorySize;
        }
        var progressElement = panel.querySelector('.robot-progress');
        if (progressElement) {
            progressElement.classList.toggle('robot-progress-over', programSize > memorySize);
        }
        if (barElement) {
            barElement.style.width = percent.toFixed(1) + '%';
        }
    }

    function robotApplyBlockReason(panel) {
        var nameInput = panel.querySelector('input[name^="robotName"]');
        if (nameInput) {
            var robotName = nameInput.value.trim();
            if (!robotName || robotName.length > 15 || !/^[A-Za-z0-9_]+$/.test(robotName)) {
                return 'Invalid robot name.';
            }
        }
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        if (programSelect && memorySelect) {
            var selectedProgram = programSelect.options[programSelect.selectedIndex];
            var selectedMemory = memorySelect.options[memorySelect.selectedIndex];
            var programSize = parseInt(selectedProgram.getAttribute('data-compiled-size') || '0', 10);
            var memorySize = parseInt(selectedMemory.getAttribute('data-memory-capacity') || '0', 10);
            if (memorySize > 0 && programSize > memorySize) {
                return 'Not enough memory available.';
            }
        }
        return null;
    }

    function updateRobotProgramHint(panel) {
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var hint = panel.querySelector('.robot-program-hint');
        if (!programSelect || !hint) {
            return;
        }
        var programOption = programSelect.options[programSelect.selectedIndex];
        var hasError = programOption.getAttribute('data-has-compile-error') === '1';
        hint.hidden = !hasError;
        if (hasError) {
            var link = hint.querySelector('a');
            if (link) {
                link.href = 'editCode?nextProgramSourceId=' + encodeURIComponent(programOption.value);
            }
        }
    }

    function updateRobotApplyState(panel) {
        if (!panel) {
            return;
        }
        var reason = robotApplyBlockReason(panel);
        var applyButton = panel.querySelector('.robot-btn-primary');
        var hint = panel.querySelector('.robot-action-hint');
        if (applyButton) {
            applyButton.disabled = !!reason;
            if (reason) {
                applyButton.setAttribute('title', reason);
            } else {
                applyButton.removeAttribute('title');
            }
        }
        if (hint) {
            if (reason) {
                hint.textContent = reason;
                hint.hidden = false;
            } else {
                hint.textContent = '';
                hint.hidden = true;
            }
        }
        updateRobotProgramHint(panel);
        updateRobotQuickLinks(panel);
        updateRobotMemoryPreview(panel);
        updateRobotDirtyState(panel);
    }

    function attachRobotPreviewListeners(panel) {
        var programSelect = panel.querySelector('select[name^="programSourceId"]');
        var memorySelect = panel.querySelector('select[name^="memoryModuleId"]');
        var nameInput = panel.querySelector('input[name^="robotName"]');
        if (programSelect) {
            programSelect.addEventListener('change', function() {
                updateRobotApplyState(panel);
            });
        }
        if (memorySelect) {
            memorySelect.addEventListener('change', function() {
                updateRobotApplyState(panel);
            });
        }
        if (nameInput) {
            nameInput.addEventListener('input', function() {
                updateRobotApplyState(panel);
            });
        }
    }

    function robotUrlId() {
        var search = window.location.search;
        if (!search) {
            return null;
        }
        var params = search.substring(1).split('&');
        for (var index = 0; index < params.length; index += 1) {
            var pair = params[index].split('=');
            if (decodeURIComponent(pair[0]) === 'robotId' && pair[1]) {
                return decodeURIComponent(pair[1]);
            }
        }
        return null;
    }

    var preferredRobotId = robotUrlId();
    if (preferredRobotId && document.querySelector('.robot-config-panel[data-robot-id="' + preferredRobotId + '"]')) {
        selectRobot(preferredRobotId, false);
    } else {
        var firstCard = document.querySelector('.robot-fleet-card');
        if (firstCard) {
            selectRobot(firstCard.getAttribute('data-robot-id'), false);
        }
    }

    var fleetCards = document.querySelectorAll('.robot-fleet-card');
    for (var fleetIndex = 0; fleetIndex < fleetCards.length; fleetIndex += 1) {
        fleetCards[fleetIndex].addEventListener('click', function(event) {
            var robotId = event.currentTarget.getAttribute('data-robot-id');
            var activePanel = document.querySelector('.robot-config-panel-active');
            if (activePanel
                && activePanel.getAttribute('data-robot-id') !== robotId
                && isPanelDirty(activePanel)) {
                var nameInput = activePanel.querySelector('input[name^="robotName"]');
                var robotName = nameInput && nameInput.value.trim() ? nameInput.value.trim() : 'this robot';
                robominerConfirm('Discard unsaved changes to ' + robotName + '?', function(confirmed) {
                    if (!confirmed) {
                        return;
                    }
                    restorePanelBaseline(activePanel);
                    updateRobotApplyState(activePanel);
                    selectRobot(robotId);
                });
                return;
            }
            selectRobot(robotId);
        });
    }

    var resetButtons = document.querySelectorAll('.robot-reset-btn');
    for (var resetIndex = 0; resetIndex < resetButtons.length; resetIndex += 1) {
        resetButtons[resetIndex].addEventListener('click', function(event) {
            var panel = event.target.closest('.robot-config-panel');
            if (!panel) {
                return;
            }
            restorePanelBaseline(panel);
            updateRobotApplyState(panel);
        });
    }

    window.addEventListener('beforeunload', function(event) {
        if (allowPageUnload) {
            return;
        }
        var panels = document.querySelectorAll('.robot-config-panel');
        for (var unloadIndex = 0; unloadIndex < panels.length; unloadIndex += 1) {
            if (isPanelDirty(panels[unloadIndex])) {
                event.preventDefault();
                event.returnValue = '';
                return;
            }
        }
    });

    var panels = document.querySelectorAll('.robot-config-panel');
    for (var panelIndex = 0; panelIndex < panels.length; panelIndex += 1) {
        attachRobotPreviewListeners(panels[panelIndex]);
    }

    function confirmRobotApply(event) {
        var panel = null;
        if (event.submitter) {
            panel = event.submitter.closest('.robot-config-panel');
        }
        if (!panel) {
            panel = document.querySelector('.robot-config-panel-active');
        }
        if (!panel) {
            return;
        }
        var applyButton = panel.querySelector('.robot-btn-primary');
        if (applyButton && applyButton.disabled) {
            event.preventDefault();
            return;
        }
        var nameInput = panel.querySelector('input[name^="robotName"]');
        var robotName = nameInput ? nameInput.value.trim() : 'this robot';
        if (robotForm.getAttribute('data-robominer-confirmed') === '1') {
            robotForm.removeAttribute('data-robominer-confirmed');
            return;
        }
        event.preventDefault();
        robominerConfirm('Apply configuration changes to ' + robotName + '?', function(confirmed) {
            if (!confirmed) {
                return;
            }
            allowPageUnload = true;
            capturePanelBaseline(panel);
            updateRobotApplyState(panel);
            var robotId = panel.getAttribute('data-robot-id');
            if (robotId) {
                robotForm.action = 'robot?robotId=' + encodeURIComponent(robotId);
            }
            robotForm.setAttribute('data-robominer-confirmed', '1');
            if (typeof robotForm.requestSubmit === 'function') {
                robotForm.requestSubmit(event.submitter || undefined);
            } else {
                robotForm.submit();
            }
        });
    }

    var robotForm = document.getElementById('robotForm');
    if (robotForm) {
        robotForm.addEventListener('submit', confirmRobotApply);
    }
})();
</script>"#,
    );
    body.push_str("</div>");

    layout("RoboMiner - Robot", "robot", &username, hud, &body)
}

fn render_robot_summary(body: &mut String, robot_count: usize, pending_count: usize) {
    body.push_str(r#"<section class="robot-summary" aria-label="Robot fleet">"#);
    body.push_str(r#"<div class="robot-summary-heading">"#);
    body.push_str(r#"<h1 class="robot-page-title">Robot workshop</h1>"#);
    body.push_str("</div>");
    body.push_str(r#"<ul class="robot-summary-list">"#);
    body.push_str(&format!(
        r#"<li class="robot-summary-item"><span class="robot-summary-label">Robots</span><span class="robot-summary-value">{}</span></li>"#,
        robot_count
    ));
    body.push_str(&format!(
        r#"<li class="robot-summary-item"><span class="robot-summary-label">Pending changes</span><span class="robot-summary-value">{}</span></li>"#,
        pending_count
    ));
    body.push_str("</ul></section>");
}

fn render_robot_claim_banner(body: &mut String, state: &RobotPageState) {
    body.push_str(&crate::html::render_claimed_ore_rewards_banner(
        "robot-claim-banner",
        &state.claimed_results,
        true,
    ));
}

fn render_robot_message(body: &mut String, state: &RobotPageState) {
    let Some(message) = &state.message else {
        return;
    };
    let banner_class = if message.starts_with("Unable") {
        "robot-banner robot-banner-error"
    } else {
        "robot-banner robot-banner-success"
    };
    body.push_str(&format!(
        r#"<p class="{banner_class}">{}</p>"#,
        escape_html(message)
    ));
}

fn render_robot_fleet_card(
    body: &mut String,
    robot: &robominer_db::RobotConfigStateRecord,
    program_sources: &[robominer_db::ProgramSourceRecord],
    active: bool,
) {
    let active_class = if active {
        " robot-fleet-card-active"
    } else {
        ""
    };
    let program_size = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .map(|program_source| program_source.compiled_size)
        .unwrap_or(0);
    let status_class = if robot.change_pending {
        "robot-fleet-status-pending"
    } else {
        "robot-fleet-status-ready"
    };
    let status_label = if robot.change_pending {
        "Pending"
    } else {
        "Ready"
    };

    body.push_str(&format!(
        r#"<button type="button" class="robot-fleet-card{active_class}" data-robot-id="{}">"#,
        robot.robot_id
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-heading"><span class="robot-fleet-name">{}</span><span class="robot-fleet-status {status_class}">{status_label}</span></span>"#,
        escape_html(&robot.robot_name)
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-highlights"><span>Ore {}</span><span>Mining {}</span><span>CPU {}</span></span>"#,
        robot.max_ore, robot.mining_speed, robot.cpu_speed
    ));
    body.push_str(&format!(
        r#"<span class="robot-fleet-memory">Memory {}/{}</span>"#,
        program_size, robot.memory_size
    ));
    body.push_str("</button>");
}

fn render_robot_config_panel(
    body: &mut String,
    robot: &robominer_db::RobotConfigStateRecord,
    active: bool,
    program_sources: &[robominer_db::ProgramSourceRecord],
    part_asset_map: &HashMap<i64, Vec<&robominer_db::RobotConfigPartAssetStateRecord>>,
) {
    let active_class = if active {
        " robot-config-panel-active"
    } else {
        ""
    };
    let hidden_attr = if active { "" } else { " hidden" };
    let disabled_attr = if active { "" } else { " disabled" };
    let program_size = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .map(|program_source| program_source.compiled_size)
        .unwrap_or(0);
    let memory_percent = robot_memory_percent(program_size, robot.memory_size);
    let memory_overflow = program_size > robot.memory_size;
    let selected_program_has_error = program_sources
        .iter()
        .find(|program_source| program_source.id == robot.program_source_id)
        .is_some_and(|program_source| !program_source.error_description.is_empty());
    let program_hint_hidden = if active && selected_program_has_error {
        ""
    } else {
        " hidden"
    };

    let change_pending_attr = if robot.change_pending {
        r#" data-change-pending="true""#
    } else {
        r#" data-change-pending="false""#
    };
    body.push_str(&format!(
        r#"<div class="robot-config-panel{active_class}" id="robotDetails{}" data-robot-id="{}"{change_pending_attr}{hidden_attr}>"#,
        robot.robot_id, robot.robot_id
    ));
    body.push_str(&format!(
        r#"<input type="hidden" name="robotId" value="{}"{disabled_attr}/>"#,
        robot.robot_id
    ));
    body.push_str(r#"<header class="robot-config-header">"#);
    body.push_str(&format!(
        r#"<div><h2 class="robot-config-title">{}</h2><p class="robot-config-subtitle">Configure parts and program source</p></div>"#,
        escape_html(&robot.robot_name)
    ));
    if robot.change_pending {
        body.push_str(r#"<span class="robot-status-badge robot-status-pending">Changes pending</span>"#);
    } else {
        body.push_str(r#"<span class="robot-status-badge robot-status-ready">Ready to apply</span>"#);
        body.push_str(r#"<span class="robot-status-badge robot-status-dirty" hidden>Unsaved changes</span>"#);
    }
    body.push_str("</header>");
    body.push_str(&format!(
        r#"<div class="robot-quick-links"><a class="robot-quick-link robot-quick-link-edit-program" href="editCode?nextProgramSourceId={}">Edit program</a><a class="robot-quick-link" href="helpProgramTips">Programming tips</a><a class="robot-quick-link" href="helpMechanics">Mechanics guide</a><a class="robot-quick-link" href="miningQueue?robotId={}">Mining queue</a><a class="robot-quick-link" href="shop">Shop parts</a></div>"#,
        robot.program_source_id, robot.robot_id
    ));

    body.push_str(r#"<section class="robot-config-section"><h3 class="robot-section-title">Identity</h3><div class="robot-field-grid">"#);
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">Name</span><input type="text" id="robotName{}" name="robotName{}" class="robot-text-input" value="{}" maxlength="15" pattern="[A-Za-z0-9_]{{1,15}}" placeholder="1 to 15 characters, only letters and numbers" required{disabled_attr} /></label>"#,
        robot.robot_id,
        robot.robot_id,
        escape_html(&robot.robot_name)
    ));
    body.push_str(&format!(
        r#"<label class="robot-field"><span class="robot-field-label">Source code</span><select id="programSourceId{}" name="programSourceId{}" class="tableitem robot-select"{disabled_attr}>"#,
        robot.robot_id, robot.robot_id
    ));
    for program_source in program_sources {
        let compile_error_attr = if program_source.error_description.is_empty() {
            String::new()
        } else {
            r#" data-has-compile-error="1""#.to_string()
        };
        body.push_str(&format!(
            r#"<option value="{}" data-compiled-size="{}"{compile_error_attr}{}>{}</option>"#,
            program_source.id,
            program_source.compiled_size,
            selected_attr(robot.program_source_id == program_source.id),
            escape_html(&program_source.source_name)
        ));
    }
    body.push_str("</select>");
    body.push_str(&format!(
        r#"<p class="robot-program-hint"{program_hint_hidden}>Selected program has a compile error. <a class="robot-program-hint-link" href="editCode?nextProgramSourceId={}">Fix in editor</a></p>"#,
        robot.program_source_id
    ));
    body.push_str("</label></div></section>");

    body.push_str(r#"<section class="robot-config-section"><h3 class="robot-section-title">Parts</h3><div class="robot-field-grid">"#);
    render_robot_part_select(
        body,
        "Ore container",
        "oreContainerId",
        robot.robot_id,
        1,
        robot.ore_container_id,
        &robot.ore_container_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Mining unit",
        "miningUnitId",
        robot.robot_id,
        2,
        robot.mining_unit_id,
        &robot.mining_unit_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Battery",
        "batteryId",
        robot.robot_id,
        3,
        robot.battery_id,
        &robot.battery_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Memory module",
        "memoryModuleId",
        robot.robot_id,
        4,
        robot.memory_module_id,
        &robot.memory_module_name,
        part_asset_map,
        true,
        disabled_attr,
        Some(robot.memory_size),
    );
    render_robot_part_select(
        body,
        "CPU",
        "cpuId",
        robot.robot_id,
        5,
        robot.cpu_id,
        &robot.cpu_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Engine",
        "engineId",
        robot.robot_id,
        6,
        robot.engine_id,
        &robot.engine_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    render_robot_part_select(
        body,
        "Ore scanner",
        "oreScannerId",
        robot.robot_id,
        7,
        robot.ore_scanner_id,
        &robot.ore_scanner_name,
        part_asset_map,
        false,
        disabled_attr,
        None,
    );
    body.push_str("</div></section>");

    body.push_str(r#"<section class="robot-config-section"><h3 class="robot-section-title">Performance</h3>"#);
    body.push_str(r#"<div class="robot-stat-highlights">"#);
    push_robot_highlight(body, "Ore cap", robot.max_ore, " units");
    push_robot_highlight(body, "Mining", robot.mining_speed, " u/c");
    push_robot_highlight(body, "CPU", robot.cpu_speed, " i/c");
    push_robot_highlight(body, "Cycles", robot.max_turns, "");
    body.push_str("</div>");
    render_robot_memory_progress(body, program_size, robot.memory_size, memory_percent, memory_overflow);
    body.push_str(r#"<dl class="robot-stat-grid">"#);
    add_robot_stat_entry(body, "Forward speed:", format!("{:.2} s/c", robot.forward_speed));
    add_robot_stat_entry(body, "Backward speed:", format!("{:.2} s/c", robot.backward_speed));
    add_robot_stat_entry(body, "Rotate speed:", format!("{} d/c", robot.rotate_speed));
    add_robot_stat_entry(body, "Size:", format!("{:.2} s", robot.robot_size));
    add_robot_stat_entry(body, "Scan time:", format!("{} cycles", robot.scan_time));
    add_robot_stat_entry(body, "Scan distance:", format!("{}", robot.scan_distance));
    add_robot_stat_entry(body, "Recharge time:", format_period(robot.recharge_time));
    body.push_str("</dl></section>");

    body.push_str(r#"<div class="robot-apply">"#);
    body.push_str(&render_robot_apply_action(robot, program_sources, disabled_attr));
    body.push_str("</div></div>");
}

fn render_robot_memory_progress(
    body: &mut String,
    program_size: i32,
    memory_size: i32,
    percent: f64,
    overflow: bool,
) {
    let overflow_class = if overflow {
        " robot-progress-over"
    } else {
        ""
    };
    body.push_str(&format!(r#"<div class="robot-progress{overflow_class}">"#));
    body.push_str(&format!(
        r#"<div class="robot-progress-heading"><span>Memory used</span><span class="robot-progress-value">{}/{}</span></div>"#,
        program_size, memory_size
    ));
    body.push_str(r#"<div class="robot-progress-track" aria-hidden="true">"#);
    body.push_str(&format!(
        r#"<div class="robot-progress-bar" style="width: {percent:.1}%"></div>"#
    ));
    body.push_str("</div></div>");
}

fn render_robot_apply_action(
    robot: &robominer_db::RobotConfigStateRecord,
    program_sources: &[robominer_db::ProgramSourceRecord],
    disabled_attr: &str,
) -> String {
    let block_reason = robot_apply_block_reason(robot, program_sources);
    let mut html = String::from(r#"<div class="robot-apply-actions">"#);
    html.push_str(&robot_apply_button(block_reason, disabled_attr));
    html.push_str(r#"<button type="button" class="robot-btn robot-btn-secondary robot-reset-btn" hidden>Reset changes</button></div>"#);
    if let Some(reason) = block_reason {
        html.push_str(&format!(
            r#"<p class="robot-action-hint">{}</p>"#,
            escape_html(reason)
        ));
    } else {
        html.push_str(r#"<p class="robot-action-hint" hidden></p>"#);
    }
    html.push_str(
        r#"<p class="robot-apply-helper">Apply queues part and program changes for this robot.</p>"#,
    );
    html
}

fn robot_apply_button(
    block_reason: Option<&str>,
    disabled_attr: &str,
) -> String {
    let title_attr = block_reason
        .map(|reason| format!(r#" title="{}""#, escape_html(reason)))
        .unwrap_or_default();
    if block_reason.is_some() || !disabled_attr.is_empty() {
        format!(
            r#"<button type="submit" class="robot-btn robot-btn-primary" disabled{title_attr}{disabled_attr}>Apply changes</button>"#
        )
    } else {
        r#"<button type="submit" class="robot-btn robot-btn-primary">Apply changes</button>"#.to_string()
    }
}


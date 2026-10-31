// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use serde::Serialize;

use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize)]
pub struct Layout {
    name: String,
    short_description: Option<String>,
    description: String,
}

pub struct LayoutManager {
    layouts: Vec<Layout>,
}

impl LayoutManager {
    pub fn instance() -> &'static LayoutManager {
        static MANAGER: OnceLock<LayoutManager> = OnceLock::new();
        MANAGER.get_or_init(|| Self {
            layouts: system_layouts(),
        })
    }

    /// Get keyboard layouts
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    /// Get current keyboard layouts
    pub fn layout(&self) -> &Layout {
        x11::xkb_layout(&self.layouts)
    }

    /// Set keyboard layout
    pub fn set_layout(&self, layout: &str) -> bool {
        let idx = self
            .layouts
            .iter()
            .enumerate()
            .find(|(_, l)| l.name == layout)
            .map(|(i, _)| i)
            .unwrap();
        x11::switch_xkb_layout(&self.layouts[idx])
    }
}

fn system_layouts() -> Vec<Layout> {
    match xkb_data::keyboard_layouts() {
        Ok(layout_list) => layout_list
            .layouts()
            .iter()
            .flat_map(|l| {
                let layout = &l.config_item;
                let mut layouts = vec![Layout {
                    name: layout.name.clone(),
                    short_description: layout.short_description.clone(),
                    description: layout.description.clone(),
                }];
                if let Some(v) = l.variants() {
                    layouts.extend(
                        v.iter()
                            .map(|v| Layout {
                                name: [&layout.name, v.name()].join("@"),
                                short_description: layout.short_description.clone(),
                                description: layout.description.clone(),
                            })
                            .collect::<Vec<Layout>>(),
                    );
                }
                layouts
            })
            .collect(),
        Err(e) => {
            log::error!("Failed to get all keyborad layouts: {e}");
            Vec::new()
        }
    }
}

mod x11 {
    use x11rb::{
        connection::Connection,
        protocol::xproto::{AtomEnum, ConnectionExt, PropMode, change_property, get_property},
    };

    use super::Layout;

    const LAYOUT_VARIANT_SEP: &str = "@";

    fn xkb_rmlvo() -> Vec<String> {
        if let Ok((conn, screen_id)) = x11rb::connect(None)
            && let Ok(request) = conn.intern_atom(false, b"_XKB_RULES_NAMES")
            && let Ok(atom_reply) = request.reply()
        {
            let reply = get_property(
                &conn,
                false,
                conn.setup().roots[screen_id].root,
                atom_reply.atom,
                AtomEnum::STRING,
                0,
                1024,
            )
            .unwrap()
            .reply()
            .unwrap();
            let prop = String::from_utf8_lossy(&reply.value);
            let rmlvo: Vec<_> = prop.split_terminator('\0').map(str::to_string).collect();
            if rmlvo.len() == 5 {
                return rmlvo;
            } else {
                log::error!("Target window property is not an valid rmlvo: {rmlvo:?}");
            }
        }

        log::error!("Failed to get root_window keyboard layout");
        vec!["".to_string(); 5]
    }

    pub(super) fn xkb_layout(layouts: &[Layout]) -> &Layout {
        if let Some(lv) = xkb_rmlvo().get(2..4) {
            let layout = if lv.iter().any(|s| s.is_empty()) {
                lv.first().unwrap().to_string()
            } else {
                lv.join(LAYOUT_VARIANT_SEP)
            };
            if let Some(l) = layouts.iter().find(|l| l.name == layout) {
                return l;
            };
        }

        layouts.first().unwrap()
    }

    pub(super) fn switch_xkb_layout(layout: &Layout) -> bool {
        let old = xkb_rmlvo();
        let rmlvo = if old.iter().all(String::is_empty) {
            log::error!("old keyboard layout is not an valid rmlvo: {old:?}");
            return false;
        } else {
            let lv: Vec<_> = layout.name.split(LAYOUT_VARIANT_SEP).collect();
            let (l, v) = if lv.len() == 2 {
                (lv[0], lv[1])
            } else {
                (lv[0], "")
            };
            [&old[0], &old[1], l, v, &old[4], ""].join("\0")
        };
        if let Ok((conn, screen_id)) = x11rb::connect(None)
            && let Ok(request) = conn.intern_atom(false, b"_XKB_RULES_NAMES")
            && let Ok(atom_reply) = request.reply()
        {
            change_property(
                &conn,
                PropMode::REPLACE,
                conn.setup().roots[screen_id].root,
                atom_reply.atom,
                AtomEnum::STRING,
                8,
                rmlvo.len() as _,
                rmlvo.as_bytes(),
            )
            .is_ok_and(|reply| reply.check().is_ok())
        } else {
            false
        }
    }
}

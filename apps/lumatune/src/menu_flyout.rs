use crate::{MenuFlyoutBehavior, MenuFlyoutItem, MenuFlyoutItemKind};
use slint::{Model, ModelExt, ModelRc};

pub fn install(behavior: MenuFlyoutBehavior<'_>) {
    behavior.on_next_item(next_item);
    behavior.on_has_icons(|items| any_row(items, |item| item.icon.size().width > 0));
    behavior.on_has_checks(|items| {
        any_row(items, |item| {
            matches!(
                item.kind,
                MenuFlyoutItemKind::Toggle | MenuFlyoutItemKind::Radio
            )
        })
    });
    behavior.on_update_checks(update_checks);
}

fn any_row(items: ModelRc<MenuFlyoutItem>, predicate: impl Fn(MenuFlyoutItem) -> bool) -> bool {
    items.model_tracker().track_row_count_changes();
    (0..items.row_count()).any(|index| items.row_data_tracked(index).is_some_and(&predicate))
}

fn next_item(items: ModelRc<MenuFlyoutItem>, current: i32, forward: bool) -> i32 {
    let count = items.row_count();
    if count == 0 {
        return -1;
    }

    let current = usize::try_from(current).ok().filter(|index| *index < count);
    let first = match (current, forward) {
        (Some(index), true) => (index + 1) % count,
        (Some(index), false) => (index + count - 1) % count,
        (None, true) => 0,
        (None, false) => count - 1,
    };

    (0..count)
        .map(|offset| {
            let offset = if forward { offset } else { count - offset };
            (first + offset) % count
        })
        .find(|&index| {
            items
                .row_data(index)
                .is_some_and(|item| !item.disabled && item.kind != MenuFlyoutItemKind::Separator)
        })
        .map_or(-1, |index| index as i32)
}

fn update_checks(items: ModelRc<MenuFlyoutItem>, index: i32) {
    let Ok(index) = usize::try_from(index) else {
        return;
    };
    let Some(mut item) = items.row_data(index).filter(|item| !item.disabled) else {
        return;
    };

    match item.kind {
        MenuFlyoutItemKind::Toggle => {
            item.checked = !item.checked;
            items.set_row_data(index, item);
        }
        MenuFlyoutItemKind::Radio => {
            for (row, mut peer) in items.iter().enumerate() {
                if peer.kind == MenuFlyoutItemKind::Radio && peer.group == item.group {
                    let checked = row == index;
                    if peer.checked != checked {
                        peer.checked = checked;
                        items.set_row_data(row, peer);
                    }
                }
            }
        }
        MenuFlyoutItemKind::Command | MenuFlyoutItemKind::Separator => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::VecModel;

    #[test]
    fn skips_disabled_and_separator_rows_in_both_directions() {
        let items = ModelRc::new(VecModel::from(vec![
            MenuFlyoutItem::default(),
            MenuFlyoutItem {
                kind: MenuFlyoutItemKind::Separator,
                ..Default::default()
            },
            MenuFlyoutItem {
                disabled: true,
                ..Default::default()
            },
            MenuFlyoutItem::default(),
        ]));

        assert_eq!(next_item(items.clone(), -1, true), 0);
        assert_eq!(next_item(items.clone(), -1, false), 3);
        assert_eq!(next_item(items.clone(), 0, true), 3);
        assert_eq!(next_item(items.clone(), 3, false), 0);
        assert_eq!(next_item(items.clone(), 3, true), 0);
        assert_eq!(next_item(items, 0, false), 3);
    }

    #[test]
    fn toggle_and_radio_selection_update_the_model() {
        let radio = |group: &str, checked| MenuFlyoutItem {
            kind: MenuFlyoutItemKind::Radio,
            group: group.into(),
            checked,
            ..Default::default()
        };
        let items = ModelRc::new(VecModel::from(vec![
            radio("sort", true),
            radio("sort", false),
            radio("view", true),
            MenuFlyoutItem {
                kind: MenuFlyoutItemKind::Toggle,
                ..Default::default()
            },
        ]));

        update_checks(items.clone(), 1);
        update_checks(items.clone(), 3);
        let checked: Vec<_> = items.iter().map(|item| item.checked).collect();
        assert_eq!(checked, [false, true, true, true]);
    }
}

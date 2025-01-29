use bevy::{color::palettes::css, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .init_resource::<RecipeBook>()
        .init_resource::<CraftingGrid>()
        .insert_resource(Inventory {
            items: vec![ItemId::Wood.into(), ItemId::Planks.into()],
        })
        .insert_resource(SelectedItemSlot(0))
        .insert_resource(UiScale(3.))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_inventory_ui, update_bench_ui))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, inventory: Res<Inventory>) {
    commands.spawn(Camera2d);

    // Root node
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            // Crafting bench node
            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    // Crafting table texture
                    parent.spawn(ImageNode::new(
                        asset_server.load("textures/crafting_table.png"),
                    ));

                    // Crafting grid
                    parent
                        .spawn(Node {
                            top: Val::Px(16.),
                            left: Val::Px(29.),
                            display: Display::Grid,
                            position_type: PositionType::Absolute,
                            grid_template_columns: vec![GridTrack::px(18.); 3],
                            grid_template_rows: vec![GridTrack::px(18.); 3],
                            grid_auto_flow: GridAutoFlow::Row,
                            ..default()
                        })
                        .with_children(|parent| {
                            for slot in 0..9 {
                                parent
                                    .spawn((
                                        CraftingInputSlot(slot),
                                        Node {
                                            margin: UiRect::all(Val::Px(2.)),
                                            ..default()
                                        },
                                    ))
                                    .observe(on_bench_slot_click);
                            }
                        });

                    // Crafting output
                    parent
                        .spawn(Node {
                            top: Val::Px(30.),
                            left: Val::Px(119.),
                            position_type: PositionType::Absolute,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                CraftingOutputSlot,
                                Node {
                                    width: Val::Px(20.),
                                    height: Val::Px(20.),
                                    margin: UiRect::all(Val::Px(3.)),
                                    ..default()
                                },
                            ));

                            parent.spawn((
                                CraftingOutputCount,
                                Node {
                                    right: Val::Px(0.),
                                    bottom: Val::Px(0.),
                                    margin: UiRect::horizontal(Val::Px(2.)),
                                    position_type: PositionType::Absolute,
                                    ..default()
                                },
                                Text::default(),
                                TextFont {
                                    font_size: 10.,
                                    ..default()
                                },
                            ));
                        });

                    // Inventory slot (quick bar)
                    parent
                        .spawn(Node {
                            top: Val::Px(141.),
                            left: Val::Px(7.),
                            position_type: PositionType::Absolute,
                            ..default()
                        })
                        .with_children(|parent| {
                            for (slot, item) in inventory.items.iter().enumerate() {
                                parent
                                    .spawn((
                                        InventorySlot(slot),
                                        Node {
                                            width: Val::Px(14.),
                                            height: Val::Px(14.),
                                            margin: UiRect::all(Val::Px(2.)),
                                            ..default()
                                        },
                                        ImageNode::new(asset_server.load(item.id.icon_path())),
                                    ))
                                    .observe(on_item_slot_click);
                            }
                        });
                });
        });
}

fn on_item_slot_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    inventory_slots: Query<&InventorySlot>,
) {
    let &InventorySlot(slot) = inventory_slots.get(click.entity()).unwrap();
    commands.insert_resource(SelectedItemSlot(slot));
}

fn on_bench_slot_click(
    click: Trigger<Pointer<Click>>,
    inventory: Res<Inventory>,
    mut crafting_grid: ResMut<CraftingGrid>,
    selected_item: Option<Res<SelectedItemSlot>>,
    crafting_inputs: Query<&CraftingInputSlot>,
) {
    let item = match click.button {
        PointerButton::Primary => selected_item.map(|slot| inventory.items[slot.0].id),
        PointerButton::Secondary => None,
        _ => return,
    };

    let &CraftingInputSlot(slot) = crafting_inputs.get(click.entity()).unwrap();
    crafting_grid.0[slot] = item;
}

fn update_inventory_ui(
    mut commands: Commands,
    selected_item: Res<SelectedItemSlot>,
    inventory_slots: Query<(Entity, &InventorySlot)>,
) {
    for (entity, &InventorySlot(slot)) in &inventory_slots {
        if selected_item.0 == slot {
            commands.entity(entity).insert(Outline {
                width: Val::Px(2.),
                offset: Val::Px(-1.),
                color: css::BLACK.into(),
            });
        } else {
            commands.entity(entity).remove::<Outline>();
        }
    }
}

fn update_bench_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    recipe_book: Res<RecipeBook>,
    crafting_grid: Res<CraftingGrid>,
    crafting_inputs: Query<(Entity, &CraftingInputSlot)>,
    crafting_output: Single<Entity, With<CraftingOutputSlot>>,
    mut output_count: Single<&mut Text, With<CraftingOutputCount>>,
) {
    if !crafting_grid.is_changed() {
        return;
    }

    for (entity, &CraftingInputSlot(slot)) in &crafting_inputs {
        if let Some(item) = crafting_grid.0[slot] {
            commands
                .entity(entity)
                .insert(ImageNode::new(asset_server.load(item.icon_path())));
        } else {
            commands.entity(entity).remove::<ImageNode>();
        }
    }

    match recipe_book.get_matching(crafting_grid.0).map(|r| r.output) {
        Some((output, n)) => {
            commands
                .entity(*crafting_output)
                .insert(ImageNode::new(asset_server.load(output.icon_path())));

            output_count.0 = n.to_string();
        }
        None => {
            commands.entity(*crafting_output).remove::<ImageNode>();
            output_count.0 = String::new();
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct Item {
    id: ItemId,
}

impl From<ItemId> for Item {
    fn from(id: ItemId) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ItemId {
    Wood,
    Planks,
    Stick,
}

impl ItemId {
    pub fn icon_path(&self) -> &'static str {
        match self {
            ItemId::Wood => "icons/crafting/wood.png",
            ItemId::Planks => "icons/crafting/planks.png",
            ItemId::Stick => "icons/crafting/stick.png",
        }
    }
}

#[derive(Resource, Debug)]
struct Inventory {
    items: Vec<Item>,
}

#[derive(Component, Debug)]
struct InventorySlot(usize);

#[derive(Resource, Debug)]
struct SelectedItemSlot(usize);

#[derive(Resource, Debug, Default)]
struct CraftingGrid([Option<ItemId>; 9]);

#[derive(Component, Debug)]
struct CraftingInputSlot(usize);

#[derive(Component, Debug)]
struct CraftingOutputSlot;

#[derive(Component, Debug)]
struct CraftingOutputCount;

#[derive(Debug)]
struct Recipe {
    size: UVec2,
    inputs: Vec<Option<ItemId>>,
    output: (ItemId, u32),
}

impl Recipe {
    fn to_grid(&self, offset: impl Into<UVec2>) -> [Option<ItemId>; 9] {
        let offset = offset.into();
        let (ox, oy) = (offset.x as usize, offset.y as usize);

        let mut grid = [None; 9];
        for y in 0..self.size.y as usize {
            for x in 0..self.size.x as usize {
                grid[(y + oy) * 3 + (x + ox)] = self.inputs[y * self.size.x as usize + x];
            }
        }
        grid
    }
}

#[derive(Resource, Debug)]
struct RecipeBook {
    recipes: Vec<Recipe>,
}

impl RecipeBook {
    fn get_matching(&self, mats: [Option<ItemId>; 9]) -> Option<&Recipe> {
        for recipe in &self.recipes {
            // Try fitting a recipe in all possible positions
            for oy in 0..=3 - recipe.size.y as usize {
                for ox in 0..=3 - recipe.size.x as usize {
                    if mats == recipe.to_grid([ox as u32, oy as u32]) {
                        return Some(recipe);
                    }
                }
            }
        }

        None
    }
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self {
            recipes: vec![
                Recipe {
                    size: [1, 1].into(),
                    inputs: vec![ItemId::Wood.into()],
                    output: (ItemId::Planks, 4),
                },
                Recipe {
                    size: [1, 2].into(),
                    inputs: vec![ItemId::Planks.into(), ItemId::Planks.into()],
                    output: (ItemId::Stick, 4),
                },
            ],
        }
    }
}

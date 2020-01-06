
use amethyst::{
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
        Camera,
        ImageFormat,
        SpriteRender,
        SpriteSheet,
        SpriteSheetFormat,
        Texture,
    },
    ecs::prelude::*,
    utils::application_root_dir,
    core::transform::{TransformBundle, Transform},
    core::math::Vector2,
    assets::{PrefabLoaderSystemDesc, Handle, AssetStorage, Loader},
};

use specs_physics::{
    nphysics::{
        math::{Vector, Velocity},
        object::{ColliderDesc, BodyPartHandle, RigidBodyDesc, BodyStatus},
    },
    PhysicsBundle,
    BodyComponent,
    ColliderComponent,
    ncollide::shape::{Cuboid, ShapeHandle},
};

fn main() -> amethyst::Result<()>{
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let display_config_path = assets_dir.join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?
        .with_bundle(PhysicsBundle::<f32, Transform>::new(Vector::y() * -9.81, &[]))?;

    let mut game = Application::build(assets_dir, Playing::new())?
        .build(game_data)?;
    game.run();

    Ok(())
}

struct Playing<'a, 'b> {
    fixed_dispatcher: Dispatcher<'a, 'b>,
    sprite_sheet_handle: Option<Handle<SpriteSheet>>,
}

impl<'a, 'b> Playing<'a, 'b> {
    pub fn new() -> Self {
        let fixed_dispatcher = DispatcherBuilder::new().build();
        Self {
            fixed_dispatcher,
            sprite_sheet_handle: None,
        }
    }
}

impl<'a, 'b> SimpleState for Playing<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        self.sprite_sheet_handle.replace(load_sprite_sheet(world));

        initialize_crate(world, self.sprite_sheet_handle.clone().unwrap());
        initialize_camera(world);
    }

    fn fixed_update(&mut self, data: StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        self.fixed_dispatcher.dispatch(data.world);
        Trans::None
    }
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();
    let texture_storage = world.read_resource::<AssetStorage<Texture>>();
    let texture_handle = loader.load(
        "logo.png",
        ImageFormat::default(),
        (),
        &texture_storage,
    );

    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "logo.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store
    )
}

fn initialize_crate(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>) {
    build_crate(world, sprite_sheet_handle.clone(), 30.0, 70.0, BodyStatus::Dynamic);
    build_crate(world, sprite_sheet_handle, 0.0, 10.0, BodyStatus::Static);
}

fn build_crate(world: &mut World, sprite_sheet_handle: Handle<SpriteSheet>, x: f32, y: f32, status: BodyStatus) {
    let mut local_transform = Transform::default();
    local_transform.set_translation_xyz(x, y, 0.0);

    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle,
        sprite_number: 0
    };

    let entity = world
        .create_entity()
        .with(sprite_render)
        .with(local_transform)
        .with(BodyComponent::new(
            RigidBodyDesc::new()
                .translation(Vector2::new(x, y))
                .status(status)
                .velocity(Velocity::new(Vector2::new(0.0, 0.0), 0.0))
                .build()
            )
        )
        .build();

    let shape = ShapeHandle::new(Cuboid::new(Vector::new(16.0, 16.0)));

    // there may be a better way to do this, idk
    world.exec(|mut colliders: WriteStorage<ColliderComponent<f32>>| {
        colliders.insert(entity,
            ColliderComponent(
                ColliderDesc::new(shape)
                    .density(0.05)
                    .build(BodyPartHandle(entity, 0))
            )
        ).unwrap();
    });
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(0.0, 50.0, 1.0);
    let camera = Camera::standard_2d(100.0, 100.0);

    world
        .create_entity()
        .with(transform)
        .with(camera)
        .build();
}
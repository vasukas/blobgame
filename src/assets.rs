use crate::common::*;

#[derive(Default)]
pub struct MyAssets {
    pub crystal: Handle<Image>,
    pub glow: Handle<Image>,
}

//

pub struct MyAssetsPlugin;

impl Plugin for MyAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyAssets>()
            .add_startup_system(load_assets);
    }
}

fn load_assets(mut assets: ResMut<MyAssets>, server: Res<AssetServer>) {
    assets.crystal = server.load("sprites/crystal.png");
    assets.glow = server.load("sprites/glow.png");
}

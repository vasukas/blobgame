use crate::common::*;
use bevy::asset::{Asset, AssetPath};

/// Resource - tracks asset loading
#[derive(Default)]
pub struct Loading {
    assets: Vec<HandleUntyped>,
}

impl Loading {
    pub fn add<'a, T: Asset, P: Into<AssetPath<'a>>>(
        &mut self, server: &AssetServer, path: P,
    ) -> Handle<T> {
        let handle = server.load(path);
        self.assets.push(handle.clone_untyped());
        handle
    }

    pub fn add_n<T: Asset>(
        &mut self, server: &AssetServer, path: &str, count: usize, extension: &str,
    ) -> Vec<Handle<T>> {
        (0..count)
            .into_iter()
            .map(|index| self.add(server, &format!("{}{}.{}", path, index, extension)))
            .collect()
    }

    pub fn complete(&self) -> bool {
        self.assets.is_empty()
    }
}

//

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Loading>()
            .add_system_to_stage(CoreStage::PostUpdate, check_state);
    }
}

fn check_state(mut load: ResMut<Loading>, server: Res<AssetServer>) {
    load.assets.retain(|handle| {
        use bevy::asset::LoadState::*;
        let state = server.get_load_state(handle);
        match state {
            Loading => true,
            Loaded | Failed => false,
            NotLoaded | Unloaded => {
                log::debug!("Invalid asset state: {:?}", state);
                false
            }
        }
    });
}

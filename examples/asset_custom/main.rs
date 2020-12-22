use amethyst::{
    assets::{
        Asset, AssetProcessorSystemBundle, AssetStorage, Format, Handle, Loader, ProcessingState,
        ProgressCounter, Source,
    },
    error::{format_err, Error, ResultExt},
    prelude::*,
    utils::application_root_dir,
};
use log::info;
use ron::de::Deserializer;
use serde::{Deserialize, Serialize};

/// Custom asset representing an energy blast.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct EnergyBlast {
    /// How much HP to subtract.
    pub hp_damage: u32,
    /// How much MP to subtract.
    pub mp_damage: u32,
}

/// A handle to a `EnergyBlast` asset.
pub type EnergyBlastHandle = Handle<EnergyBlast>;

impl Asset for EnergyBlast {
    const NAME: &'static str = "my_crate::EnergyBlast";
    type Data = Self;
}

impl From<EnergyBlast> for Result<ProcessingState<EnergyBlast>, Error> {
    fn from(energy_blast: EnergyBlast) -> Result<ProcessingState<EnergyBlast>, Error> {
        Ok(ProcessingState::Loaded(energy_blast))
    }
}

pub struct LoadingState {
    /// Tracks loaded assets.
    progress_counter: ProgressCounter,
    /// Handle to the energy blast.
    energy_blast_handle: Option<EnergyBlastHandle>,
}

/// Format for loading from `.mylang` files.
#[derive(Clone, Copy, Debug, Default)]
pub struct MyLangFormat;

impl<D> Format<D> for MyLangFormat
where
    D: for<'a> Deserialize<'a> + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        "MyLang"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<D, Error> {
        let mut deserializer = Deserializer::from_bytes(&bytes)
            .with_context(|_| format_err!("Failed deserializing MyLang file"))?;
        let val = D::deserialize(&mut deserializer)
            .with_context(|_| format_err!("Failed parsing MyLang file"))?;
        deserializer
            .end()
            .with_context(|_| format_err!("Failed parsing MyLang file"))?;

        Ok(val)
    }
}

#[derive(Debug)]
struct CodeSource;

impl Source for CodeSource {
    fn modified(&self, _path: &str) -> Result<u64, Error> {
        Ok(0)
    }
    fn load(&self, _path: &str) -> Result<Vec<u8>, Error> {
        let bytes = b"EnergyBlast(hp_damage: 10, mp_damage: 10)".to_vec();
        Ok(bytes)
    }
}

impl SimpleState for LoadingState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        {
            let mut loader = data.resources.get_mut::<Loader>().unwrap();
            loader.add_source("code_source", CodeSource);
        }

        let loader = data.resources.get::<Loader>().unwrap();

        let energy_blast_handle = loader.load_from(
            "energy_blast.mylang",
            self::MyLangFormat,
            "code_source",
            &mut self.progress_counter,
            &data.resources.get::<AssetStorage<EnergyBlast>>().unwrap(),
        );

        self.energy_blast_handle = Some(energy_blast_handle);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData>) -> SimpleTrans {
        if self.progress_counter.is_complete() {
            let energy_blast_assets = data.resources.get::<AssetStorage<EnergyBlast>>().unwrap();
            let energy_blast = energy_blast_assets
                .get(
                    self.energy_blast_handle
                        .as_ref()
                        .expect("Expected energy_blast_handle to be set."),
                )
                .expect("Expected energy blast to be loaded.");
            info!("Loaded energy blast: {:?}", energy_blast);
            Trans::Quit
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let app_root = application_root_dir()?;
    let assets_dir = app_root.join("example/asset_custom/assets");

    let mut builder = DispatcherBuilder::default();
    builder.add_bundle(AssetProcessorSystemBundle::<EnergyBlast>::default());

    let game = Application::build(
        assets_dir,
        LoadingState {
            progress_counter: ProgressCounter::new(),
            energy_blast_handle: None,
        },
    )?
    .build(builder)?;

    game.run();
    Ok(())
}

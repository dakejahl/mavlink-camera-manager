use super::types::*;
use super::video_source_gst::VideoSourceGst;
use super::video_source_local::VideoSourceLocal;
use super::video_source_redirect::VideoSourceRedirect;
use tracing::*;

pub trait VideoSource {
    fn name(&self) -> &String;
    fn source_string(&self) -> &str;
    fn formats(&self) -> Vec<Format>;
    fn set_control_by_name(&self, control_name: &str, value: i64) -> std::io::Result<()>;
    fn set_control_by_id(&self, control_id: u64, value: i64) -> std::io::Result<()>;
    fn control_value_by_name(&self, control_name: &str) -> std::io::Result<i64>;
    fn control_value_by_id(&self, control_id: u64) -> std::io::Result<i64>;
    fn controls(&self) -> Vec<Control>;
    fn is_valid(&self) -> bool;
    fn is_shareable(&self) -> bool;
}

pub trait VideoSourceAvailable {
    fn cameras_available() -> Vec<VideoSourceType>;
}

pub fn cameras_available() -> Vec<VideoSourceType> {
    return [
        &VideoSourceLocal::cameras_available()[..],
        &VideoSourceGst::cameras_available()[..],
        &VideoSourceRedirect::cameras_available()[..],
    ]
    .concat();
}

pub fn get_video_source(source_string: &str) -> Result<VideoSourceType, std::io::Error> {
    let cameras = cameras_available();

    if let Some(camera) = cameras
        .iter()
        .find(|source| source.inner().source_string() == source_string)
    {
        return Ok(camera.clone());
    }

    let sources_available: Vec<String> = cameras
        .iter()
        .map(|source| source.inner().source_string().to_string())
        .collect();

    return Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "The source string '{source_string}' does not exist, the available options are: {sources_available:?}."
        ),
    ));
}

pub fn set_control(source_string: &str, control_id: u64, value: i64) -> std::io::Result<()> {
    let camera = get_video_source(source_string)?;
    debug!("Set camera ({source_string}) control ({control_id}) value ({value}).");
    return camera.inner().set_control_by_id(control_id, value);
}

pub fn reset_controls(source_string: &str) -> Result<(), Vec<std::io::Error>> {
    let camera = get_video_source(source_string);
    if let Err(error) = camera {
        return Err(vec![error]);
    }
    let camera = camera.unwrap();

    debug!("Resetting all controls of camera ({source_string}).",);

    let mut errors: Vec<std::io::Error> = Default::default();
    for control in camera.inner().controls() {
        if control.state.is_inactive {
            continue;
        }

        let default_value = match &control.configuration {
            ControlType::Bool(bool) => bool.default,
            ControlType::Slider(slider) => slider.default,
            ControlType::Menu(menu) => menu.default,
        };

        if let Err(error) = camera
            .inner()
            .set_control_by_id(control.id, default_value as i64)
        {
            let error_message = format!(
                "Error when trying to reset control '{}' (id {}). Error: {}.",
                control.name,
                control.id,
                error.to_string()
            );
            errors.push(std::io::Error::new(error.kind(), error_message));
        }
    }
    if errors.is_empty() {
        return Ok(());
    }

    error!("{errors:#?}");
    return Err(errors);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_test() {
        println!("{:#?}", cameras_available());
    }
}

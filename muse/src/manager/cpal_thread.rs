use crate::sampler::Sample;
use crossbeam::channel::Receiver;

pub fn copy_samples(
    samples: &Receiver<Sample>,
    data: &mut [f32],
    format: &cpal::StreamConfig,
) -> Result<(), anyhow::Error> {
    for sample in data.chunks_mut(format.channels as usize) {
        let generated_sample = samples.recv()?;

        match format.channels {
            1 => {
                sample[0] =
                    cpal::Sample::from(&((generated_sample.left + generated_sample.right) / 2.0))
            }
            2 => {
                sample[0] = cpal::Sample::from(&generated_sample.left);
                sample[1] = cpal::Sample::from(&generated_sample.right);
            }
            _ => panic!("Unsupported number of channels {}", format.channels),
        }
    }

    Ok(())
}

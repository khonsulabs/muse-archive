use crate::sampler::Sample;
use cpal::{traits::EventLoopTrait, Sample as CpalSample};
use crossbeam::channel::Receiver;

pub fn run(samples: Receiver<Sample>, event_loop: cpal::EventLoop, format: cpal::Format) -> ! {
    event_loop.run(move |id, result| {
        let data = match result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", id, err);
                return;
            }
        };

        match data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::U16(buffer),
            } => {
                let _ = copy_samples(&samples, buffer, &format);
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(buffer),
            } => {
                let _ = copy_samples(&samples, buffer, &format);
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(buffer),
            } => {
                let _ = copy_samples(&samples, buffer, &format);
            }
            _ => (),
        }
    });
}

fn copy_samples<S>(
    samples: &Receiver<Sample>,
    mut buffer: cpal::OutputBuffer<S>,
    format: &cpal::Format,
) -> Result<(), anyhow::Error>
where
    S: CpalSample,
{
    for sample in buffer.chunks_mut(format.channels as usize) {
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

import numpy as np
from cython_vst_loader.vst_host import VstHost
from cython_vst_loader.vst_loader_wrapper import allocate_float_buffer, get_float_buffer_as_list, free_buffer
from cython_vst_loader.vst_plugin import VstPlugin

from config import Config


# class to wrap the functionality of the RNNoise VST plugin
class RNNoiseVST(object):
    # frame size to use for processing
    _frame_size: int
    # instance of the VST plugin
    _rnnoise: VstPlugin

    def __init__(self, frame_size: int, sample_rate: int, config: Config):
        self._frame_size = frame_size

        host = VstHost(sample_rate, self._frame_size)  # initialize the VST host
        self._rnnoise = VstPlugin("rnnoise_mono.dll".encode(), host)  # load the RNNoise VST plugin

        self._set_parameters(  # set the plugin parameters from the config
            config.VADThreshold,
            config.VADGracePeriod,
            config.retroactiveVADGracePeriod
        )

    # process a single audio frame
    def process_frame(self, frame: bytes) -> bytes:
        input_array = np.frombuffer(frame, dtype=np.int16)  # convert the input frame from bytes to a numpy array
        clipped_input = np.clip(input_array.astype(np.float32) / 32768, -1.0, 1.0)  # clip the input audio to [-1, 1]

        input_pointer = numpy_array_to_pointer(clipped_input)  # convert the numpy array to a pointer
        output_pointer = allocate_float_buffer(self._frame_size, 0)  # allocate memory for the output buffer

        self._rnnoise.process_replacing([input_pointer], [output_pointer], self._frame_size)  # process the input frame

        output_buffer = get_float_buffer_as_list(output_pointer, self._frame_size)  # get the output buffer as a list
        # rescale the output buffer to int16 and convert to bytes
        rescaled_output = np.round(np.clip(output_buffer, -1.0, 1.0) * 32767).astype(np.int16)

        free_buffer(output_pointer)  # free the memory of the output buffer
        return rescaled_output.tobytes()  # return the processed audio frame as bytes

    # Set the plugin parameters
    def _set_parameters(self, vt: float, vgp: float, rvgp: float) -> None:
        self._rnnoise.set_parameter_value(0, vt)  # VAD Threshold
        self._rnnoise.set_parameter_value(1, vgp)  # VAD Grace Period
        self._rnnoise.set_parameter_value(2, rvgp)  # Retroactive VAD Grace Period


def numpy_array_to_pointer(numpy_array: np.ndarray) -> int:
    if numpy_array.ndim != 1:
        raise Exception("expected a 1d numpy array here")

    pointer, _ = numpy_array.__array_interface__["data"]
    return pointer

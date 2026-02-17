#include <AudioUnit/AudioUnit.h>

typedef void (*AudioCallback)(void* user_data, float* buffer, uint32_t frame_count);

typedef struct {
  void* user_data;
  AudioCallback callback;
  AudioUnit output_unit;
} AudioEngine;

static OSStatus renderCallback(
  void* inRefCon,
  AudioUnitRenderActionFlags* ioActionFlags,
  const AudioTimeStamp* inTimeStamp,
  uint32_t inBusNumber,
  uint32_t inNumberFrames,
  AudioBufferList* ioData
) {
  AudioEngine* engine = (AudioEngine*)inRefCon;

  // Assume 'mono' for now.
  float* buffer = (float*)ioData->mBuffers[0].mData;
  engine->callback(engine->user_data, buffer, inNumberFrames);

  /* for (uint32_t ch = 0; ch < ioData->mNumberBuffers; ch++) { */
  /*   float* buffer = (float*)ioData->mBuffers[ch].mData; */
  /*   engine->callback(engine->user_data, buffer, inNumberFrames); */
  /* } */

  return noErr;
}

AudioEngine* audio_engine_new(void* user_data, AudioCallback callback, double sample_rate) {
  printf("[C] New engine\n");
  AudioEngine* engine = malloc(sizeof(AudioEngine));
  memset(engine, 0, sizeof(AudioEngine));

  engine->user_data = user_data;
  engine->callback = callback;

  // Describe output unit
  AudioComponentDescription desc = {0};
  desc.componentType = kAudioUnitType_Output;
  desc.componentSubType = kAudioUnitSubType_DefaultOutput;
  desc.componentManufacturer = kAudioUnitManufacturer_Apple;

  AudioComponent comp = AudioComponentFindNext(NULL, &desc);
  OSStatus status = AudioComponentInstanceNew(comp, &engine->output_unit);
  if (status != noErr || !engine->output_unit) {
    free(engine);
    return NULL;
  }

  // Setup stream format
  AudioStreamBasicDescription format = {0};
  format.mSampleRate = sample_rate;
  format.mFormatID = kAudioFormatLinearPCM;
  format.mFormatFlags =
    kAudioFormatFlagIsFloat | kAudioFormatFlagIsNonInterleaved | kAudioFormatFlagIsPacked;
  format.mBytesPerPacket = sizeof(float);
  format.mFramesPerPacket = 1;
  format.mBytesPerFrame = sizeof(float);
  format.mBitsPerChannel = 32;
  format.mChannelsPerFrame = 1;  // mono

  AudioUnitSetProperty(
    engine->output_unit,
    kAudioUnitProperty_StreamFormat,
    kAudioUnitScope_Input,
    0,
    &format,
    sizeof(format)
  );

  // Set render callback
  AURenderCallbackStruct cb = {0};
  cb.inputProc = renderCallback;
  cb.inputProcRefCon = engine;

  AudioUnitSetProperty(
    engine->output_unit,
    kAudioUnitProperty_SetRenderCallback,
    kAudioUnitScope_Input,
    0,
    &cb,
    sizeof(cb)
  );

  AudioUnitInitialize(engine->output_unit);

  return engine;
}

void audio_engine_start(AudioEngine* engine) {
  printf("[C] Start engine\n");
  AudioOutputUnitStart(engine->output_unit);
}

void audio_engine_stop(AudioEngine* engine) {
  printf("[C] Stop engine\n");
  AudioOutputUnitStop(engine->output_unit);
}

void audio_engine_free(AudioEngine* engine) {
  if (!engine) return;
  printf("[C] Free engine\n");
  AudioUnitUninitialize(engine->output_unit);
  AudioComponentInstanceDispose(engine->output_unit);
  free(engine);
}

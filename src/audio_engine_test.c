#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

typedef void (*AudioCallback)(void* user_data, int n);

typedef struct {
  void* user_data;
  AudioCallback callback;
} AudioEngine;

AudioEngine* audio_engine_new(void* user_data, AudioCallback callback) {
  AudioEngine* engine = malloc(sizeof(AudioEngine));
  engine->user_data = user_data;
  engine->callback = callback;
  return engine;
}

void audio_engine_start(AudioEngine* engine) {
  printf("[C] Start engine\n");
  int count = 0;
  for (;;) {
    engine->callback(engine->user_data, count++);
    if (count == 3) break;
    sleep(1);
  }
}

void audio_engine_stop(AudioEngine* engine) {
  (void)engine;
  printf("[C] Stop engine\n");
}

void audio_engine_free(AudioEngine* engine) {
  printf("[C] Free engine\n");
  if (!engine) return;
  free(engine);
}

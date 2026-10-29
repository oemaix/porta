/*
 * Copyright 2026 oemaix
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#include "porta_app.h"

void app_main(void)
{
  porta_app_t app;

  porta_app_init(&app);
  /* This image is not linked yet. When idf.py is in the shell, the panel
     task calls porta_app_panel_step and the audio task calls
     porta_app_audio_step. porta_board_i2s_allowed stays false until the
     AXP192 current limit is a measured value, so this entry does not
     start the codec or Wi-Fi. */
  (void)app;
}

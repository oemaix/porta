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

#include "porta_seam.h"

void porta_queue_init(porta_queue_t *queue)
{
  queue->head = 0;
  queue->count = 0;
}

bool porta_queue_push(porta_queue_t *queue, porta_command_t command)
{
  uint8_t tail;

  if (queue->count >= PORTA_QUEUE_CAP) {
    return false;
  }
  tail = (uint8_t)((queue->head + queue->count) % PORTA_QUEUE_CAP);
  queue->items[tail] = command;
  queue->count++;
  return true;
}

bool porta_queue_pop(porta_queue_t *queue, porta_command_t *command)
{
  if (queue->count == 0) {
    return false;
  }
  *command = queue->items[queue->head];
  queue->head = (uint8_t)((queue->head + 1) % PORTA_QUEUE_CAP);
  queue->count--;
  return true;
}

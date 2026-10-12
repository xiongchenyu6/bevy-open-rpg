/* Persistent US virtual keyboard for an owned Wayland test compositor.
 * Uses normal compositor input; never reads application state.
 * Uploads one xkbcommon US keymap, then sends fixed Linux evdev keycodes.
 */
#define _GNU_SOURCE
#include "virtual-keyboard.h"
#include <poll.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>

static struct wl_seat *seat;
static struct zwp_virtual_keyboard_manager_v1 *manager;
static struct wl_display *display;
static struct zwp_virtual_keyboard_v1 *keyboard;
struct binding {
  const char *name;
  uint32_t code;
  uint32_t pressed_at;
  int held;
};
static struct binding bindings[] = {
    {"W", 17, 0, 0},     {"A", 30, 0, 0},     {"S", 31, 0, 0},
    {"D", 32, 0, 0},     {"Space", 57, 0, 0}, {"Return", 28, 0, 0},
    {"Escape", 1, 0, 0}, {"Up", 103, 0, 0},   {"Down", 108, 0, 0},
    {"Left", 105, 0, 0}, {"Right", 106, 0, 0}};
static uint32_t tick(void) {
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return (uint32_t)(t.tv_sec * 1000 + t.tv_nsec / 1000000);
}
static void global(void *data, struct wl_registry *registry, uint32_t id,
                   const char *name, uint32_t version) {
  if (!strcmp(name, "wl_seat") && !seat)
    seat = wl_registry_bind(registry, id, &wl_seat_interface, 1);
  if (!strcmp(name, "zwp_virtual_keyboard_manager_v1"))
    manager = wl_registry_bind(registry, id,
                               &zwp_virtual_keyboard_manager_v1_interface, 1);
}
static void removed(void *data, struct wl_registry *registry, uint32_t id) {}
static struct binding *lookup(const char *name) {
  if (!strcasecmp(name, "Enter"))
    name = "Return";
  if (!strcasecmp(name, "Esc"))
    name = "Escape";
  for (size_t i = 0; i < sizeof(bindings) / sizeof(bindings[0]); i++)
    if (!strcasecmp(name, bindings[i].name))
      return &bindings[i];
  return NULL;
}
static void send_key(struct binding *key, int pressed) {
  if (key->held == pressed)
    return;
  if (!pressed) {
    uint32_t age = tick() - key->pressed_at;
    if (age < 150)
      usleep((150 - age) * 1000);
  }
  zwp_virtual_keyboard_v1_key(keyboard, tick(), key->code,
                              pressed ? WL_KEYBOARD_KEY_STATE_PRESSED
                                      : WL_KEYBOARD_KEY_STATE_RELEASED);
  key->held = pressed;
  if (pressed)
    key->pressed_at = tick();
  wl_display_roundtrip(display);
  // Let clients observe a released frame before a subsequent tap.
  if (!pressed)
    usleep(150000);
}
static int upload_us_keymap(void) {
  struct xkb_context *context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
  if (!context)
    return 0;
  const struct xkb_rule_names names = {
      .rules = "evdev", .model = "pc105", .layout = "us"};
  struct xkb_keymap *map =
      xkb_keymap_new_from_names(context, &names, XKB_KEYMAP_COMPILE_NO_FLAGS);
  if (!map) {
    xkb_context_unref(context);
    return 0;
  }
  char *text = xkb_keymap_get_as_string(map, XKB_KEYMAP_FORMAT_TEXT_V1);
  if (!text) {
    xkb_keymap_unref(map);
    xkb_context_unref(context);
    return 0;
  }
  size_t size = strlen(text) + 1;
  int fd = memfd_create("love-rpg-fixed-us-keymap", MFD_CLOEXEC);
  int ok = fd >= 0 && ftruncate(fd, size) == 0;
  if (ok) {
    void *buffer = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
    if (buffer == MAP_FAILED)
      ok = 0;
    else {
      memcpy(buffer, text, size);
      zwp_virtual_keyboard_v1_keymap(keyboard, WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1,
                                     fd, size);
      zwp_virtual_keyboard_v1_modifiers(keyboard, 0, 0, 0, 0);
      wl_display_roundtrip(display);
      munmap(buffer, size);
    }
  }
  if (fd >= 0)
    close(fd);
  if (ok) {
    for (size_t i = 0; i < sizeof(bindings) / sizeof(bindings[0]); i++) {
      char symbol[64];
      const xkb_keysym_t *symbols = NULL;
      int count = xkb_keymap_key_get_syms_by_level(map, bindings[i].code + 8, 0,
                                                   0, &symbols);
      if (count > 0) {
        xkb_keysym_get_name(symbols[0], symbol, sizeof symbol);
        printf("KEYMAP %s evdev=%u symbol=%s\n", bindings[i].name,
               bindings[i].code, symbol);
      }
    }
  }
  free(text);
  xkb_keymap_unref(map);
  xkb_context_unref(context);
  return ok;
}
int main(void) {
  setvbuf(stdin, NULL, _IONBF, 0);
  setvbuf(stdout, NULL, _IONBF, 0);
  display = wl_display_connect(NULL);
  if (!display) {
    perror("Wayland connect");
    return 1;
  }
  struct wl_registry *registry = wl_display_get_registry(display);
  const struct wl_registry_listener listener = {global, removed};
  wl_registry_add_listener(registry, &listener, NULL);
  wl_display_roundtrip(display);
  if (!seat || !manager) {
    fprintf(stderr, "No virtual keyboard manager/seat on owned compositor\n");
    return 2;
  }
  keyboard =
      zwp_virtual_keyboard_manager_v1_create_virtual_keyboard(manager, seat);
  if (!upload_us_keymap()) {
    fprintf(stderr, "Failed to create/upload fixed US keymap\n");
    return 3;
  }
  puts("READY fixed US keymap uploaded once; tap KEY / hold KEY MS / press KEY "
       "/ release KEY / quit");
  char line[128], command[16], name[32];
  while (1) {
    struct pollfd input = {STDIN_FILENO, POLLIN, 0};
    int status = poll(&input, 1, 100);
    if (status > 0) {
      if (!fgets(line, sizeof line, stdin))
        break;
      if (!strncmp(line, "quit", 4))
        break;
      unsigned duration = 200;
      int count = sscanf(line, "%15s %31s %u", command, name, &duration);
      struct binding *key = count >= 2 ? lookup(name) : NULL;
      if (!key) {
        puts("ERROR supported keys: W A S D Space Return Escape Up Down Left "
             "Right");
        continue;
      }
      if (!strcmp(command, "press"))
        send_key(key, 1);
      else if (!strcmp(command, "release"))
        send_key(key, 0);
      else if (!strcmp(command, "tap") || !strcmp(command, "hold")) {
        if (!strcmp(command, "hold") && count < 3) {
          puts("ERROR hold KEY MS requires milliseconds");
          continue;
        }
        if (duration < 150)
          duration = 150;
        if (duration > 60000) {
          puts("ERROR maximum hold is 60000ms");
          continue;
        }
        send_key(key, 1);
        usleep(duration * 1000);
        send_key(key, 0);
      } else {
        puts("ERROR command: tap / hold / press / release / quit");
        continue;
      }
      printf("SENT %s", line);
    }
    if (wl_display_roundtrip(display) < 0)
      break;
  }
  for (size_t i = 0; i < sizeof(bindings) / sizeof(bindings[0]); i++)
    send_key(&bindings[i], 0);
  zwp_virtual_keyboard_v1_destroy(keyboard);
  wl_display_roundtrip(display);
  wl_display_disconnect(display);
  return 0;
}

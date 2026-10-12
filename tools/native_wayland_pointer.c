/* Persistent normal Wayland virtual pointer for an owned headless compositor.
 * Uses compositor input only; it never reads or mutates application state.
 * Keep this process alive so clients retain pointer capability between
 * commands.
 */
#include "virtual-pointer.h"
#include <poll.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>
static struct wl_seat *seat;
static struct zwlr_virtual_pointer_manager_v1 *manager;
static void global(void *data, struct wl_registry *reg, uint32_t id,
                   const char *name, uint32_t version) {
  if (!strcmp(name, "wl_seat") && !seat)
    seat = wl_registry_bind(reg, id, &wl_seat_interface, 1);
  if (!strcmp(name, "zwlr_virtual_pointer_manager_v1"))
    manager =
        wl_registry_bind(reg, id, &zwlr_virtual_pointer_manager_v1_interface,
                         version > 2 ? 2 : version);
}
static void removed(void *d, struct wl_registry *r, uint32_t n) {}
static uint32_t tick(void) {
  struct timespec t;
  clock_gettime(CLOCK_MONOTONIC, &t);
  return t.tv_sec * 1000 + t.tv_nsec / 1000000;
}
int main(void) {
  struct wl_display *display = wl_display_connect(NULL);
  if (!display) {
    perror("connect");
    return 1;
  }
  struct wl_registry *reg = wl_display_get_registry(display);
  const struct wl_registry_listener listener = {global, removed};
  wl_registry_add_listener(reg, &listener, NULL);
  wl_display_roundtrip(display);
  if (!seat || !manager) {
    fprintf(stderr, "missing owned seat/virtual pointer manager\n");
    return 2;
  }
  struct zwlr_virtual_pointer_v1 *pointer =
      zwlr_virtual_pointer_manager_v1_create_virtual_pointer(manager, seat);
  wl_display_roundtrip(display);
  setvbuf(stdin, NULL, _IONBF, 0);
  setvbuf(stdout, NULL, _IONBF, 0);
  puts("READY persistent virtual pointer; commands: move X Y, press, release, "
       "click, quit");
  char line[128];
  while (1) {
    struct pollfd fd = {STDIN_FILENO, POLLIN, 0};
    if (poll(&fd, 1, 100) > 0) {
      if (!fgets(line, sizeof line, stdin))
        break;
      unsigned x, y;
      if (sscanf(line, "move %u %u", &x, &y) == 2)
        zwlr_virtual_pointer_v1_motion_absolute(pointer, tick(), x, y, 1280,
                                                720);
      else if (!strncmp(line, "press", 5))
        zwlr_virtual_pointer_v1_button(pointer, tick(), 0x110,
                                       WL_POINTER_BUTTON_STATE_PRESSED);
      else if (!strncmp(line, "release", 7))
        zwlr_virtual_pointer_v1_button(pointer, tick(), 0x110,
                                       WL_POINTER_BUTTON_STATE_RELEASED);
      else if (!strncmp(line, "click", 5)) {
        zwlr_virtual_pointer_v1_button(pointer, tick(), 0x110,
                                       WL_POINTER_BUTTON_STATE_PRESSED);
        zwlr_virtual_pointer_v1_frame(pointer);
        wl_display_roundtrip(display);
        usleep(250000);
        zwlr_virtual_pointer_v1_button(pointer, tick(), 0x110,
                                       WL_POINTER_BUTTON_STATE_RELEASED);
      } else if (!strncmp(line, "quit", 4))
        break;
      zwlr_virtual_pointer_v1_frame(pointer);
      wl_display_roundtrip(display);
      printf("SENT %s", line);
    }
    if (wl_display_roundtrip(display) < 0)
      break;
  }
  zwlr_virtual_pointer_v1_destroy(pointer);
  wl_display_roundtrip(display);
  wl_display_disconnect(display);
  return 0;
}

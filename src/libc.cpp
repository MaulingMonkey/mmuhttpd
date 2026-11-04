#include <sys/socket.h>
#include <cstdlib>

typedef int fd;

static_assert(sizeof(socklen_t) == sizeof(int));

static_assert( 3 == SO_TYPE);
static_assert(30 == SO_ACCEPTCONN);

static_assert( 1 == SOL_SOCKET);

// SO_TYPE
static_assert( 1 == SOCK_STREAM);
static_assert( 2 == SOCK_DGRAM);
static_assert( 3 == SOCK_RAW);
static_assert( 5 == SOCK_SEQPACKET);

static void type_checks() {
    int (*getsockopt)(fd, int, int, void*, socklen_t*) = &::getsockopt;
}

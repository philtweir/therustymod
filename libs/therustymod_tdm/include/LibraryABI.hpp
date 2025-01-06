#include "sys_defines.hpp"
#include "sys_assert.hpp"
#include "sys_types.hpp"
#include "Vector.hpp"

typedef bool (*confirmLoad_t)();
typedef bool (*loadPCMFromMemory_t)( const char* name, int num_channels, int bits_per_sample, int num_samples_per_sec, int objectSize, int objectMemSize, const char* subtitleDecl, byte* _nonCacheData );

typedef struct {
	void (*returnString)( const char *text );
	void (*returnFloat)( const float value );
	void (*returnInt)( const int value );
	void (*returnVector)( idVec3 const &vec );
	void (*returnBytes)( char *text );
	// void (*ReturnEntity)( void *ent ) // TODO: fix void*
} returnCallbacks_t;

class LibraryABI {
	public:
		LibraryABI(
			confirmLoad_t _confirmLoad,
			returnCallbacks_t _returnCallbacks,
			loadPCMFromMemory_t _loadPCMFromMemory
		) : confirmLoad(_confirmLoad), returnCallbacks(_returnCallbacks), loadPCMFromMemory(_loadPCMFromMemory) {}

		confirmLoad_t confirmLoad;
		returnCallbacks_t returnCallbacks;
		loadPCMFromMemory_t loadPCMFromMemory;
};

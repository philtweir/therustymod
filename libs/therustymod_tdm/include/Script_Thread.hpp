/*****************************************************************************
The Dark Mod GPL Source Code

This file is part of the The Dark Mod Source Code, originally based
on the Doom 3 GPL Source Code as published in 2011.

The Dark Mod Source Code is free software: you can redistribute it
and/or modify it under the terms of the GNU General Public License as
published by the Free Software Foundation, either version 3 of the License,
or (at your option) any later version. For details, see LICENSE.TXT.

Project: The Dark Mod (http://www.thedarkmod.com/)

******************************************************************************/

#include "Entity.hpp"
#include "Vector.hpp"

class idThread : public idClass {
        public:
            static void					ReturnString( const char *text );
            static void					ReturnFloat( const float value );
            static void					ReturnInt( const int value );
            static void					ReturnVector( idVec3 const &vec );
            static void					ReturnEntity( idEntity *ent );
};

#SingleInstance Force
#UseHook True

A_MaxHotkeysPerInterval := 200

KeysBlocked := false

F12::{
    global KeysBlocked := !KeysBlocked
    if KeysBlocked
        SoundBeep(750, 100)
    else
        SoundBeep(250, 100)
}

#HotIf KeysBlocked
*a::return
*b::return
*c::return
*d::return
*e::return
*f::return
*g::return
*h::return
*i::return
*j::return
*k::return
*l::return
*m::return
*n::return
*o::return
*p::return
*q::return
*r::return
*s::return
*t::return
*u::return
*v::return
*w::return
*x::return
*y::return
*z::return
*1::return
*2::return
*3::return
*4::return
*5::return
*6::return
*7::return
*8::return
*9::return
*0::return
*,::return
*.::return
*/::return
*;::return
*'::return
*\::return
*[::return
*]::return
*-::return
*=::return
*SC056::return
#HotIf

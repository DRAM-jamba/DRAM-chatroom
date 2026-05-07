# KAN-88 ui testing

went through all the service files and App.tsx manually,
no test runner in package.json (no jest or vitest) so 
this is basically a code review looking for bugs and 
things that dont work yet

## app navigation

servers page -> nickname page -> sessions page -> chat page

the flow makes sense. nickname gets collected before 
sessions load which is right. leaving a session sends 
you back to sessions, disconnecting sends you back to 
servers. nothing weird here.

## server stuff

getServers and addServer both call tauri properly, 
those look fine.

removeServer has a bug - it passes `id` to rust but 
the remove_server command expects `ip`. so removing 
a server will just fail silently or throw an error 
at runtime.

updateServer is still a stub. it returns fake data 
and doesnt call rust at all. theres no rename_server 
command in lib.rs yet so nothing to connect it to.

## session stuff

createSession calls tauri but expects a string back 
(the session key). the rust command returns nothing 
so this will crash when someone tries to create a session.

getSessions, addSession, joinSession, forgetSession 
all use hardcoded in-memory data. sessions disappear 
every time the app restarts since nothing is saved anywhere.

## chat stuff

everything in chatService is fake. getMessages returns 
the same hardcoded messages every time. sendMessage 
adds to a local array but doesnt actually send anything 
to the server. the whole chat tab is mocked right now.

## things worth fixing

removeServer needs to pass ip not id to rust

createSession needs to match what the rust command 
actually returns or the rust command needs to return 
the key properly

at some point sessions need to be persisted like servers 
are, otherwise users lose their session list on every restart

sendMessage should probably check if the user is actually 
connected before trying to send
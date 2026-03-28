// new tests for the new server 

const base = 'http://localhost:3000'

// still works the same
const r1 = await fetch(base + '/')
const b1 = await r1.json()
console.log('health check -', r1.status, b1)

// this route returns NotFound from error_check() in main.rs
const r2 = await fetch(base + '/error')
console.log('error route -', r2.status)

// /server/add - this one actually does something
// creates a new user, should give back auth_token and user_key
const r3 = await fetch(base + '/server/add')
const b3 = await r3.json()
console.log('add user -', r3.status, b3)

// trying to connect with the key from above
// user_key is hardcoded in the code so just using that
const r4 = await fetch(base + '/server/connect/hhee22HAM4433')
const b4 = await r4.json()
console.log('connect valid key -', r4.status, b4)

// key that doesnt exist - should be 404
const r5 = await fetch(base + '/server/connect/thiskeyisnotreal')
console.log('connect bad key -', r5.status)

// these are stubs, just checking they dont crash
const r6 = await fetch(base + '/server/leave')
const b6 = await r6.json()
console.log('server leave -', r6.status, b6)

const r7 = await fetch(base + '/session/create')
const b7 = await r7.json()
console.log('session create -', r7.status, b7)

// checking old route is actually gone now
const r8 = await fetch(base + '/api/users')
console.log('old /api/users route -', r8.status)
// expecting 404
// Run only on the real server/development machine with Node already available.
// Keys go to stdout for the operator; this script never writes or commits secrets.
import { generateKeyPairSync } from 'node:crypto';
const { privateKey } = generateKeyPairSync('ec', { namedCurve: 'prime256v1' });
const key = privateKey.export({format: 'jwk'});
const publicBytes = Buffer.concat([Buffer.from([4]), Buffer.from(key.x, 'base64url'), Buffer.from(key.y, 'base64url')]);
process.stdout.write(`VAPID_PUBLIC_KEY=${publicBytes.toString('base64url')}\nVAPID_PRIVATE_KEY=${key.d}\n`);
process.stderr.write('Keep the private key in the server environment. Set WEB_PUSH_ENABLED=true and VAPID_SUBJECT to a real operator contact. Never commit or share the private key.\n');

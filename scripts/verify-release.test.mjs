import assert from 'node:assert/strict';
import { createHash, generateKeyPairSync, sign } from 'node:crypto';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { verifyMinisign, verifyRelease } from './verify-release.mjs';

function fixture() {
  const bytes = Buffer.from('fabricated installer bytes');
  const { publicKey, privateKey } = generateKeyPairSync('ed25519');
  const keyId = Buffer.from('12345678');
  const publicPacket = Buffer.concat([
    Buffer.from('Ed'),
    keyId,
    publicKey.export({ format: 'der', type: 'spki' }).subarray(-32),
  ]);
  const publicText = `untrusted comment: test key\n${publicPacket.toString('base64')}\n`;
  const signature = sign(null, createHash('blake2b512').update(bytes).digest(), privateKey);
  const packet = Buffer.concat([Buffer.from('ED'), keyId, signature]);
  const comment = 'timestamp:123\tfile:app_1.2.3_x64-setup.exe\tversion:1.2.3';
  const global = sign(null, Buffer.concat([signature, Buffer.from(comment)]), privateKey);
  const text = `untrusted comment: test signature\n${packet.toString('base64')}\ntrusted comment: ${comment}\n${global.toString('base64')}\n`;
  const encodedKey = Buffer.from(publicText).toString('base64');
  const encodedSignature = Buffer.from(text).toString('base64');
  const asset = {
    name: 'app_1.2.3_x64-setup.exe',
    size: bytes.length,
    state: 'uploaded',
    digest: `sha256:${createHash('sha256').update(bytes).digest('hex')}`,
    browser_download_url:
      'https://github.com/example/app/releases/download/v1.2.3/app_1.2.3_x64-setup.exe',
  };
  const platform = { url: asset.browser_download_url, signature: encodedSignature };
  const manifest = {
    version: '1.2.3',
    pub_date: '2026-10-14T00:00:00Z',
    platforms: { 'windows-x86_64': { ...platform }, 'windows-x86_64-nsis': { ...platform } },
  };
  const release = {
    tag_name: 'v1.2.3',
    prerelease: false,
    assets: [asset, { name: `${asset.name}.sig`, state: 'uploaded' }],
  };
  return { bytes, encodedKey, encodedSignature, text, manifest, release, asset };
}

test('verifies both Minisign signatures and rejects modified bytes, comments and keys', () => {
  const f = fixture();
  assert.match(verifyMinisign(f.bytes, f.encodedSignature, f.encodedKey), /version:1.2.3/);
  assert.throws(() => verifyMinisign(Buffer.from('tampered'), f.encodedSignature, f.encodedKey));
  const tamperedComment = Buffer.from(f.text.replace('version:1.2.3', 'version:1.2.4')).toString(
    'base64',
  );
  assert.throws(() => verifyMinisign(f.bytes, tamperedComment, f.encodedKey));
  assert.throws(() => verifyMinisign(f.bytes, f.encodedSignature, fixture().encodedKey));
  assert.throws(() => verifyMinisign(f.bytes, 'not base64', f.encodedKey));
  assert.throws(() => verifyMinisign(f.bytes, `${f.encodedSignature}AAAA`, f.encodedKey));
});

test('rejects a legacy or mismatched algorithm and key ID', () => {
  const f = fixture();
  const lines = f.text.trim().split('\n');
  const packet = Buffer.from(lines[1], 'base64');
  for (const offset of [1, 2]) {
    const modified = Buffer.from(packet);
    modified[offset] ^= 1;
    const corrupted = [...lines];
    corrupted[1] = modified.toString('base64');
    const signature = Buffer.from(corrupted.join('\n')).toString('base64');
    assert.throws(() => verifyMinisign(f.bytes, signature, f.encodedKey));
  }
});

test('release gate verifies downloaded assets and fails closed for broken manifests', () => {
  const f = fixture();
  const dir = mkdtempSync(join(tmpdir(), 'aism-release-'));
  try {
    writeFileSync(join(dir, f.asset.name), f.bytes);
    writeFileSync(join(dir, `${f.asset.name}.sig`), f.encodedSignature);
    const check = (manifest = f.manifest, release = f.release) =>
      verifyRelease(manifest, release, 'v1.2.3', '1.2.3', dir, f.encodedKey);
    check();
    const mutations = [
      (m) => {
        m.version = '1.2.4';
      },
      (m) => {
        delete m.platforms['windows-x86_64'];
      },
      (m) => {
        m.platforms['windows-x86_64'].url = 'http://github.com/bad';
      },
      (m) => {
        m.platforms['windows-x86_64'].signature = 'AAAA';
      },
      (m) => {
        m.pub_date = 'not a date';
      },
    ];
    for (const mutate of mutations) {
      const manifest = structuredClone(f.manifest);
      mutate(manifest);
      assert.throws(() => check(manifest));
    }
    for (const change of [
      { size: 999 },
      { digest: 'sha256:bad' },
      { state: 'new' },
      { name: '../app_1.2.3_x64-setup.exe' },
    ]) {
      const release = structuredClone(f.release);
      Object.assign(release.assets[0], change);
      assert.throws(() => check(f.manifest, release));
    }
    writeFileSync(join(dir, f.asset.name), Buffer.from('corrupted installer'));
    assert.throws(() => check());
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

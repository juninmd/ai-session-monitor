import { createHash, createPublicKey, verify } from 'node:crypto';
import { readFileSync, statSync } from 'node:fs';
import { basename, join } from 'node:path';

function decode(text, length) {
  if (typeof text !== 'string' || !/^[A-Za-z0-9+/]+={0,2}$/.test(text))
    throw new Error('Invalid base64');
  const bytes = Buffer.from(text, 'base64');
  if (bytes.toString('base64') !== text || (length && bytes.length !== length))
    throw new Error('Invalid base64 length');
  return bytes;
}

// Minisign's documented ED format: Ed25519 over Blake2b-512, followed by a signature
// over the file signature and trusted comment. Legacy, un-hashed signatures are refused.
// https://jedisct1.github.io/minisign/#signature-format
export function verifyMinisign(bytes, encodedSignature, encodedPublicKey) {
  const publicLines = decode(encodedPublicKey).toString('utf8').trim().split(/\r?\n/);
  const signatureLines = decode(encodedSignature).toString('utf8').trim().split(/\r?\n/);
  if (
    publicLines.length !== 2 ||
    signatureLines.length !== 4 ||
    !publicLines[0].startsWith('untrusted comment: ') ||
    !signatureLines[0].startsWith('untrusted comment: ') ||
    !signatureLines[2].startsWith('trusted comment: ')
  )
    throw new Error('Invalid Minisign envelope');
  const publicPacket = decode(publicLines[1], 42);
  const signaturePacket = decode(signatureLines[1], 74);
  if (
    publicPacket.subarray(0, 2).toString() !== 'Ed' ||
    signaturePacket.subarray(0, 2).toString() !== 'ED' ||
    !publicPacket.subarray(2, 10).equals(signaturePacket.subarray(2, 10))
  )
    throw new Error('Invalid Minisign algorithm or key ID');
  const key = createPublicKey({
    key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), publicPacket.subarray(10)]),
    format: 'der',
    type: 'spki',
  });
  const signature = signaturePacket.subarray(10);
  const comment = signatureLines[2].slice('trusted comment: '.length);
  if (
    !verify(null, createHash('blake2b512').update(bytes).digest(), key, signature) ||
    !verify(
      null,
      Buffer.concat([signature, Buffer.from(comment)]),
      key,
      decode(signatureLines[3], 64),
    )
  )
    throw new Error('Invalid Minisign signature');
  return comment;
}

/** Verifies the actual GitHub-downloaded artifacts before the draft becomes public. */
export function verifyRelease(manifest, release, tag, version, artifactsDir, publicKey) {
  if (
    !/^\d+\.\d+\.\d+$/.test(version) ||
    tag !== `v${version}` ||
    manifest.version !== version ||
    release.tag_name !== tag ||
    release.prerelease
  )
    throw new Error('Release version or tag mismatch');
  if (!Number.isFinite(Date.parse(manifest.pub_date))) throw new Error('Invalid publication date');
  const targets = ['windows-x86_64', 'windows-x86_64-nsis'];
  if (
    !manifest.platforms ||
    Object.keys(manifest.platforms).sort().join() !== targets.sort().join()
  )
    throw new Error('Missing or unexpected updater platform');
  for (const target of targets) {
    const platform = manifest.platforms[target];
    const asset = release.assets.find(
      (item) =>
        item.browser_download_url === platform.url ||
        item.browser_download_url?.replace(/\/download\/untagged-[^/]+\//, `/download/${tag}/`) ===
          platform.url ||
        item.url === platform.url,
    );
    const url = new URL(platform.url);
    if (
      !asset ||
      url.protocol !== 'https:' ||
      !['github.com', 'api.github.com'].includes(url.hostname) ||
      url.username ||
      url.password ||
      url.search ||
      url.hash ||
      basename(asset.name) !== asset.name ||
      /[\\/]/.test(asset.name) ||
      !asset.name.includes(version) ||
      !/_x64-setup\.(exe|nsis\.zip)$/.test(asset.name) ||
      asset.state !== 'uploaded' ||
      !Number.isSafeInteger(asset.size) ||
      asset.size <= 0 ||
      asset.size > 128 * 1024 * 1024
    )
      throw new Error(`Invalid updater asset for ${target}`);
    const signatureAsset = release.assets.find((item) => item.name === `${asset.name}.sig`);
    if (signatureAsset?.state !== 'uploaded') throw new Error('Missing signature asset');
    const path = join(artifactsDir, asset.name);
    if (statSync(path).size !== asset.size) throw new Error('Installer size mismatch');
    const signature = readFileSync(`${path}.sig`, 'utf8').trim();
    if (signature !== platform.signature) throw new Error('Manifest signature mismatch');
    const bytes = readFileSync(path);
    if (asset.digest !== `sha256:${createHash('sha256').update(bytes).digest('hex')}`)
      throw new Error('Installer digest mismatch');
    const comment = verifyMinisign(bytes, signature, publicKey);
    if (!comment.split('\t').includes(`version:${version}`))
      throw new Error('Signed version mismatch');
  }
}

export function verifyReleaseFiles(args) {
  const [manifestPath, releasePath, tag, version, artifactsDir, configPath] = args;
  if (!manifestPath || !releasePath || !tag || !version || !artifactsDir || !configPath)
    throw new Error(
      'Expected manifest, release, tag, version, artifact directory and Tauri config',
    );
  const json = (path) => JSON.parse(readFileSync(path, 'utf8').replace(/^\uFEFF/, ''));
  verifyRelease(
    json(manifestPath),
    json(releasePath),
    tag,
    version,
    artifactsDir,
    json(configPath).plugins.updater.pubkey,
  );
  console.log(`Manifest, downloaded installer and Minisign signatures verified for ${tag}.`);
}

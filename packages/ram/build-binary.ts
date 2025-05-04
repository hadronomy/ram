import { globSync } from 'glob';
import { spawnSync } from 'node:child_process';
import { rmSync, copyFileSync, existsSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

const args = process.argv.slice(2);
console.log(`args: `, args);

// Extract target information from args
let targetArg = args.find(arg => arg.startsWith('--target='));
let targetPlatform = null;

if (targetArg) {
  // Format will be like --target=x86_64-unknown-linux-gnu
  targetPlatform = targetArg.split('=')[1];
  console.log(`Building for target: ${targetPlatform}`);
}

const cmd = spawnSync(
  'cargo',
  [
    '+nightly',
    'build',
    '-Z',
    'unstable-options',
    '--artifact-dir',
    './artifacts',
    '-p',
    'ramlang',
    ...args,
  ],
  {
    stdio: 'inherit', // Directly inherit stdio (preserves colors)
    env: { ...process.env, RUSTC_COLOR: 'always' }, // Force color output
    shell: true,
    cwd: __dirname,
  },
);

if (cmd.status !== 0) {
  globSync('artifacts/*', {
    absolute: true,
    cwd: __dirname,
  }).forEach((file) => {
    rmSync(file, { force: true, recursive: true });
  });

  console.error('Command failed!');
  process.exit(cmd.status);
}

// Copy binary to appropriate npm folder if target is specified
if (targetPlatform && cmd.status === 0) {
  try {
    // Map Rust target to npm package target format
    let npmTarget = mapRustTargetToNpmTarget(targetPlatform);

    if (npmTarget) {
      const { os, arch, libc } = npmTarget;
      const binaryName = 'ram' + (os === 'win32' ? '.exe' : '');
      const sourcePath = join(__dirname, 'artifacts', binaryName);
      const npmDir = join(__dirname, 'npm');
      
      // Create directory name with libc variant if applicable
      let dirName = `cli-${os}-${arch}`;
      if (os === 'linux' && libc) {
        dirName = `cli-${os}-${arch}-${libc}`;
      }
      
      const destDir = join(npmDir, dirName, 'bin');
      const destPath = join(destDir, binaryName);

      // Ensure destination directory exists
      if (!existsSync(destDir)) {
        mkdirSync(destDir, { recursive: true });
      }

      console.log(`Copying binary from ${sourcePath} to ${destPath}`);
      copyFileSync(sourcePath, destPath);
      console.log(`Successfully copied binary to npm package directory`);
    } else {
      console.warn(`Could not map Rust target "${targetPlatform}" to npm package target`);
    }
  } catch (error) {
    console.error('Error copying binary:', error);
  }
}

/**
 * Maps Rust target triple to npm package target format
 */
function mapRustTargetToNpmTarget(rustTarget: string): { os: string, arch: string, libc?: string } | null {
  // Common Rust targets and their npm equivalents
  if (rustTarget.includes('x86_64-unknown-linux-gnu')) {
    return { os: 'linux', arch: 'x64', libc: 'gnu' };
  } else if (rustTarget.includes('x86_64-unknown-linux-musl')) {
    return { os: 'linux', arch: 'x64', libc: 'musl' };
  } else if (rustTarget.includes('aarch64-unknown-linux-gnu')) {
    return { os: 'linux', arch: 'arm64', libc: 'gnu' };
  } else if (rustTarget.includes('aarch64-unknown-linux-musl')) {
    return { os: 'linux', arch: 'arm64', libc: 'musl' };
  } else if (rustTarget.includes('x86_64-apple-darwin')) {
    return { os: 'darwin', arch: 'x64' };
  } else if (rustTarget.includes('aarch64-apple-darwin')) {
    return { os: 'darwin', arch: 'arm64' };
  } else if (rustTarget.includes('x86_64-pc-windows-msvc')) {
    return { os: 'win32', arch: 'x64' };
  } else if (rustTarget.includes('aarch64-pc-windows-msvc')) {
    return { os: 'win32', arch: 'arm64' };
  }

  return null;
}

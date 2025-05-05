import { globSync } from 'glob';
import { copyFileSync, existsSync, mkdirSync, statSync, readdirSync, chmodSync } from 'node:fs';
import { dirname, basename, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));

console.log('Organizing RAM binary artifacts...');

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
  } else if (rustTarget.includes('armv7-unknown-linux-gnueabihf')) {
    return { os: 'linux', arch: 'arm', libc: 'gnu' };
  } else if (rustTarget.includes('x86_64-apple-darwin')) {
    return { os: 'darwin', arch: 'x64' };
  } else if (rustTarget.includes('aarch64-apple-darwin')) {
    return { os: 'darwin', arch: 'arm64' };
  } else if (rustTarget.includes('x86_64-pc-windows-msvc')) {
    return { os: 'win32', arch: 'x64' };
  } else if (rustTarget.includes('i686-pc-windows-msvc')) {
    return { os: 'win32', arch: 'ia32' };
  } else if (rustTarget.includes('aarch64-pc-windows-msvc')) {
    return { os: 'win32', arch: 'arm64' };
  } else if (rustTarget.includes('x86_64-unknown-freebsd')) {
    return { os: 'freebsd', arch: 'x64' };
  }

  return null;
}

/**
 * Process binary artifact and organize it into the proper npm package directory
 */
function processBinary(binaryPath: string, targetTriple: string): void {
  console.log(`Processing binary for target: ${targetTriple}`);

  const npmTarget = mapRustTargetToNpmTarget(targetTriple);
  if (!npmTarget) {
    console.error(`Unsupported target triple: ${targetTriple}`);
    return;
  }

  const { os, arch, libc } = npmTarget;
  const binaryName = 'ram' + (os === 'win32' ? '.exe' : '');
  const npmDir = join(__dirname, 'npm');

  // Create directory name with libc variant if applicable
  let dirName = `cli-${os}-${arch}`;
  if (os === 'linux' && libc) {
    dirName = `cli-${os}-${arch}-${libc}`;
  } else if (os === 'freebsd') {
    dirName = `cli-${os}-${arch}`;
  }

  const destDir = join(npmDir, dirName, 'bin');
  const destPath = join(destDir, binaryName);

  // Ensure destination directory exists
  if (!existsSync(destDir)) {
    console.log(`Creating directory: ${destDir}`);
    mkdirSync(destDir, { recursive: true });
  }

  // The binaryPath might be a directory - find the actual binary inside
  let actualBinaryPath = binaryPath;
  if (existsSync(binaryPath) && !binaryPath.endsWith(binaryName)) {
    // If path is a directory, look for the binary inside it
    if (statSync(binaryPath).isDirectory()) {
      console.log(`${binaryPath} is a directory, looking for binary inside...`);

      // Try to find the binary file within the directory
      const files = readdirSync(binaryPath);
      const binFile = files.find((file: string) => file === binaryName || file.startsWith('ram'));

      if (binFile) {
        actualBinaryPath = join(binaryPath, binFile);
        console.log(`Found binary at ${actualBinaryPath}`);
      } else {
        console.error(`Could not find binary file in directory: ${binaryPath}`);
        return;
      }
    }
  }

  console.log(`Copying binary from ${actualBinaryPath} to ${destPath}`);
  copyFileSync(actualBinaryPath, destPath);

  // Set executable permissions (rwxr-xr-x = 0755)
  console.log(`Setting executable permissions on ${destPath}`);
  chmodSync(destPath, 0o755);
}

// Find all binary files that match the pattern binary-${target}
const artifactsDir = join(__dirname, 'artifacts');
const binaryFiles = globSync('binary-*', {
  cwd: artifactsDir,
  absolute: true
});

console.log(`Found ${binaryFiles.length} binary artifacts`);

// Process each binary file
binaryFiles.forEach(binaryPath => {
  const fileName = basename(binaryPath);
  // Extract target triple from file name (format: binary-${targetTriple})
  const targetTriple = fileName.replace('binary-', '');

  if (!targetTriple) {
    console.warn(`Could not extract target triple from file name: ${fileName}`);
    return;
  }

  // Process the binary
  processBinary(binaryPath, targetTriple);
});

if (binaryFiles.length === 0) {
  console.warn('No binary artifacts found in the artifacts directory');
}

console.log('Artifact organization complete!');

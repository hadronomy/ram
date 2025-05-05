import { arch as getArch, platform as getPlatform } from "os";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

/**
 * Detects the type of libc on Linux
 * This approach looks for indicators of musl libc vs glibc
 *
 * @returns "musl" or "gnu" based on the system's libc
 */
function detectLibc(): "musl" | "gnu" {
  // Simple detection using file existence
  // The ldd command would be more reliable but requires child_process
  try {
    // Alpine Linux and other musl-based distros don't have this file
    if (fs.existsSync("/lib/ld-musl-x86_64.so.1") ||
        fs.existsSync("/lib/ld-musl-aarch64.so.1")) {
      return "musl";
    }

    // Try to detect glibc by looking for a common file
    if (fs.existsSync("/lib64/ld-linux-x86-64.so.2") ||
        fs.existsSync("/lib/ld-linux-aarch64.so.1")) {
      return "gnu";
    }

    // Fallback to glibc as it's more common
    return "gnu";
// eslint-disable-next-line no-unused-vars
  } catch (e) {
    // If we encounter any errors during detection, default to gnu
    return "gnu";
  }
}

/**
 * Attempts to find the binary by directly checking the file system
 * This is useful for global installations where import.meta.resolve might not work
 */
function findBinaryInFileSystem(packageName: string, binaryPath: string): string | null {
  // Common locations for npm global installations
  const possiblePaths = [
    // Local node_modules (most common for local installs)
    path.join(process.cwd(), 'node_modules', packageName, binaryPath),

    // Global npm installations
    // npm global prefix + node_modules + package
    ...(process.env.npm_config_prefix
      ? [path.join(process.env.npm_config_prefix, 'lib', 'node_modules', packageName, binaryPath)]
      : []),

    // Common global locations on different platforms
    ...(process.platform === 'win32'
      ? [
          // Windows global installations
          path.join(process.env.APPDATA || '', 'npm', 'node_modules', packageName, binaryPath),
          path.join(process.env.PROGRAMFILES || '', 'nodejs', 'node_modules', packageName, binaryPath),
        ]
      : [
          // Unix-like global installations
          '/usr/local/lib/node_modules/' + packageName + '/' + binaryPath,
          '/usr/lib/node_modules/' + packageName + '/' + binaryPath,
          path.join(process.env.HOME || '', '.npm-global', 'lib', 'node_modules', packageName, binaryPath),
          path.join(process.env.HOME || '', '.nvm', 'versions', 'node', process.version, 'lib', 'node_modules', packageName, binaryPath),
        ]),

    // pnpm global installations
    ...(process.env.PNPM_HOME
      ? [path.join(process.env.PNPM_HOME, 'global', 'node_modules', packageName, binaryPath)]
      : []),

    // yarn global installations
    ...(process.env.HOME
      ? [path.join(process.env.HOME, '.config', 'yarn', 'global', 'node_modules', packageName, binaryPath)]
      : []),
  ];

  // Check each path
  for (const possiblePath of possiblePaths) {
    if (fs.existsSync(possiblePath)) {
      return possiblePath;
    }
  }

  return null;
}

/**
 * Attempts to dynamically import a package and get its binary path
 */
async function tryImportPackage(packageName: string, binaryPath: string): Promise<string | null> {
  try {
    // First try to dynamically import the package
    const pkg = await import(packageName);
    if (pkg) {
      // If the package has a path property, use it
      if (pkg.path) {
        const fullPath = path.join(pkg.path, binaryPath);
        if (fs.existsSync(fullPath)) {
          return fullPath;
        }
      }
    }
  } catch {
    // Ignore import errors and continue with other methods
  }

  try {
    // Try using import.meta.resolve as a fallback
    const resolvedPath = import.meta.resolve(`${packageName}/${binaryPath}`);
    if (resolvedPath) {
      const filePath = fileURLToPath(resolvedPath);
      if (fs.existsSync(filePath)) {
        return filePath;
      }
    }
  } catch {
    // Ignore resolve errors and continue with other methods
  }

  // If dynamic import and import.meta.resolve fail, try file system search
  return findBinaryInFileSystem(packageName, binaryPath);
}

/**
 * Returns the executable path for @ramlang/cli binary
 * Works with both local and global installations
 * The naming convention is @ramlang/cli-${os}-${arch}
 * For Linux, it also includes the libc variant (gnu or musl)
 * If the platform is `win32` or `cygwin`, executable will include a `.exe` extension
 * @see https://nodejs.org/api/os.html#osarch
 * @see https://nodejs.org/api/os.html#osplatform
 */
export async function getExePath() {
  const platform = getPlatform();
  const arch = getArch();

  let os = platform as string;
  let extension = "";
  let libcSuffix = "";
  let alternativeLibc = "";

  if (platform === "win32" || platform === "cygwin") {
    os = "win32";
    extension = ".exe";
  } else if (platform === "linux") {
    // For Linux, detect and include the libc variant
    const libc = detectLibc();
    libcSuffix = `-${libc}`;

    // Set up the alternative libc for fallback
    alternativeLibc = libc === "musl" ? "-gnu" : "-musl";
  }

  // Create an array of possible package names and binary paths to try
  const packageOptions = [];

  // First try the exact OS/arch/libc match
  packageOptions.push({
    packageName: `@ramlang/cli-${os}-${arch}${libcSuffix}`,
    binaryPath: `bin/ram${extension}`
  });

  // For Linux, add fallback options
  if (platform === "linux") {
    // Try with alternative libc
    packageOptions.push({
      packageName: `@ramlang/cli-${os}-${arch}${alternativeLibc}`,
      binaryPath: `bin/ram${extension}`
    });

    // Try without libc suffix as last resort (for backwards compatibility)
    packageOptions.push({
      packageName: `@ramlang/cli-${os}-${arch}-gnu`,
      binaryPath: `bin/ram${extension}`
    });
  }

  // Try each package option until we find one that exists
  const errors = [];
  for (const { packageName, binaryPath: binPath } of packageOptions) {
    try {
      const resolvedPath = await tryImportPackage(packageName, binPath);
      if (resolvedPath) {
        return resolvedPath;
      }
    } catch (error: unknown) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      errors.push(`Failed to import ${packageName}: ${errorMessage}`);
    }
  }

  // As a last resort, try to find the binary in the artifacts directory
  // This is useful for local development scenarios
  const localArtifactPath = path.join(process.cwd(), 'artifacts', `ram${extension}`);
  if (fs.existsSync(localArtifactPath)) {
    return localArtifactPath;
  }

  // If we get here, none of the options worked
  throw new Error(
    `Couldn't find @ramlang/cli binary for ${os}-${arch}${libcSuffix}. ` +
    `Tried packages: ${packageOptions.map(opt => opt.packageName).join(", ")}. ` +
    `Errors: ${errors.join("; ")}`
  );
}

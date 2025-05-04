import { arch as getArch, platform as getPlatform } from "os";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

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
 * Attempts to resolve a package path and checks if the binary exists
 */
async function tryResolvePackage(packagePath: string): Promise<string | null> {
  try {
    const resolvedPath = await import.meta.resolve(packagePath);
    
    // Verify the binary actually exists
    const filePath = fileURLToPath(resolvedPath);
    if (fs.existsSync(filePath)) {
      return resolvedPath;
    }
    return null;
// eslint-disable-next-line no-unused-vars
  } catch (e) {
    return null;
  }
}

/**
 * Returns the executable path for @ramlang/cli located inside node_modules
 * The naming convention is @ramlang/cli-${os}-${arch}
 * For Linux, it also includes the libc variant (gnu or musl)
 * If the platform is `win32` or `cygwin`, executable will include a `.exe` extension
 * @see https://nodejs.org/api/os.html#osarch
 * @see https://nodejs.org/api/os.html#osplatform
 * @example "x/xx/node_modules/@ramlang/cli-darwin-arm64"
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

  // Create an array of possible package paths to try, in order of preference
  const packagePaths = [
    // First try the exact OS/arch/libc match
    `@ramlang/cli-${os}-${arch}${libcSuffix}/bin/ram${extension}`,
  ];
  
  // For Linux, add fallback options
  if (platform === "linux") {
    // Try with alternative libc
    packagePaths.push(`@ramlang/cli-${os}-${arch}${alternativeLibc}/bin/ram${extension}`);
    
    // Try without libc suffix as last resort (for backwards compatibility)
    packagePaths.push(`@ramlang/cli-${os}-${arch}-gnu/bin/ram${extension}`);
  }

  // Try each package path until we find one that exists
  for (const packagePath of packagePaths) {
    const resolvedPath = await tryResolvePackage(packagePath);
    if (resolvedPath) {
      return resolvedPath;
    }
  }

  // If we get here, none of the options worked
  throw new Error(
    `Couldn't find @ramlang/cli binary inside node_modules for ${os}-${arch}${libcSuffix}. ` +
    `Tried: ${packagePaths.join(", ")}`
  );
}

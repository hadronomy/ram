import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

// Define the current directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Define target platforms and architectures
const targets = [
  { os: 'darwin', arch: 'x64' },
  { os: 'darwin', arch: 'arm64' },
  { os: 'linux', arch: 'x64', libc: 'gnu' },
  { os: 'linux', arch: 'x64', libc: 'musl' },
  { os: 'linux', arch: 'arm64', libc: 'gnu' },
  { os: 'linux', arch: 'arm64', libc: 'musl' },
  { os: 'win32', arch: 'x64' },
  { os: 'win32', arch: 'arm64' },
];

// Define paths
const npmDir = join(__dirname, 'npm');
const readmeTemplatePath = join(npmDir, 'README.md.tmpl');
const packageJsonTemplatePath = join(npmDir, 'package.json.tmpl');
const rootPackageJsonPath = join(__dirname, 'package.json');

/**
 * Creates directory if it doesn't exist
 */
async function ensureDir(dir: string): Promise<void> {
  try {
    await mkdir(dir, { recursive: true });
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== 'EEXIST') {
      throw err;
    }
  }
}

/**
 * Process template string with provided values
 */
async function processTemplate(templatePath: string, values: Record<string, string>): Promise<string> {
  const content = await readFile(templatePath, 'utf-8');

  return Object.entries(values).reduce((acc, [key, value]) => {
    // Replace all occurrences of ${key} with value
    return acc.replace(new RegExp(`\\$\\{${key}\\}`, 'g'), value);
  }, content);
}

/**
 * Reads version from package.json
 */
async function getPackageVersion(): Promise<string> {
  const content = await readFile(rootPackageJsonPath, 'utf-8');
  const packageJson = JSON.parse(content);
  return packageJson.version;
}

/**
 * Updates the optionalDependencies in the main package.json file
 */
async function updateOptionalDependencies(version: string): Promise<void> {
  try {
    console.log('Updating optionalDependencies in main package.json...');

    // Read the current package.json
    const content = await readFile(rootPackageJsonPath, 'utf-8');
    const packageJson = JSON.parse(content);

    // Create new optionalDependencies object
    const optionalDependencies: Record<string, string> = {};

    // Generate package names for each target with the correct version
    for (const target of targets) {
      const { os, arch, libc } = target;

      // Create directory name with libc variant if applicable
      let packageNameSuffix = `cli-${os}-${arch}`;
      if (os === 'linux' && libc) {
        packageNameSuffix = `cli-${os}-${arch}-${libc}`;
      }

      const packageName = `@ramlang/${packageNameSuffix}`;
      optionalDependencies[packageName] = `workspace:${version}`;
    }

    // Update the packageJson object with the new optionalDependencies
    packageJson.optionalDependencies = optionalDependencies;

    // Write back to package.json with proper formatting
    await writeFile(
      rootPackageJsonPath,
      JSON.stringify(packageJson, null, 2) + '\n'
    );

    console.log('Successfully updated optionalDependencies in package.json');
  } catch (error) {
    console.error('Error updating optionalDependencies:', error);
    throw error;
  }
}

/**
 * Generates npm packages for each target
 */
async function generatePackages(): Promise<void> {
  try {
    console.log('Generating npm packages...');

    // Get version from package.json
    const version = await getPackageVersion();
    console.log(`Using package version: ${version}`);

    // Update optionalDependencies in the main package.json
    await updateOptionalDependencies(version);

    for (const target of targets) {
      const { os, arch, libc } = target;

      // Create directory name with libc variant if applicable
      let directoryName = `cli-${os}-${arch}`;
      if (os === 'linux' && libc) {
        directoryName = `cli-${os}-${arch}-${libc}`;
      }

      const packageName = `@ramlang/${directoryName}`;

      // Create target string (used in binary names)
      let targetStr = `${os}-${arch}`;
      if (os === 'linux' && libc) {
        targetStr = `${os}-${arch}-${libc}`;
      }

      const packageDir = join(npmDir, directoryName);

      console.log(`Generating package: ${packageName} in directory: ${packageDir}`);

      // Create directories
      await ensureDir(packageDir);
      await ensureDir(join(packageDir, 'bin'));

      // Create properly formatted libc field for JSON
      const libcField = libc ? `"libc": ["${libc === "gnu" ? "glibc" : "musl"}"],` : '';

      // Process templates with values using the hash template syntax matching the template files
      const values = {
        package_name: packageName,
        node_pkg: packageName,
        target: targetStr,
        os,
        arch,
        libc_field: libcField,
        version, // Add version to the values
        extension: os === 'win32' ? '.exe' : ''
      };

      // Write processed templates
      const processedReadme = await processTemplate(readmeTemplatePath, values);
      const processedPackageJson = await processTemplate(packageJsonTemplatePath, values);

      await writeFile(join(packageDir, 'README.md'), processedReadme);
      await writeFile(join(packageDir, 'package.json'), processedPackageJson);

      console.log(`Generated package: ${packageName}`);
    }

    console.log('All npm packages generated successfully!');
  } catch (error) {
    console.error('Error generating npm packages:', error);
    process.exit(1);
  }
}

// Run the generator
generatePackages();

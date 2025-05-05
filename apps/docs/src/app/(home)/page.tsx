import Link from 'next/link';
import { ArrowRight, Code, Cpu, BookOpen, Terminal } from 'lucide-react';

export default function HomePage() {
  return (
    <main className="flex flex-1 flex-col items-center">
      {/* Hero Section */}
      <section className="w-full py-20 px-4 flex flex-col items-center text-center">
        <div className="bg-gradient-to-br from-indigo-500 via-purple-500 to-pink-500 rounded-full p-6 mb-6">
          <Cpu className="w-12 h-12 text-white" />
        </div>
        <h1 className="text-5xl font-bold mb-6 h-20 bg-clip-text text-transparent bg-gradient-to-r from-indigo-500 via-purple-500 to-pink-500">
          RAM Language
        </h1>
        <p className="text-xl text-fd-muted-foreground max-w-2xl mb-8">
          A complete ecosystem for the Random Access Machine (RAM) model of computation,
          providing both a formally defined language and a robust emulator.
        </p>
        <div className="flex flex-wrap gap-4 justify-center">
          <Link
            href="/docs"
            className="px-6 py-3 rounded-lg bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-medium flex items-center gap-2 hover:opacity-90 transition-opacity"
          >
            Get Started <ArrowRight className="w-4 h-4" />
          </Link>
          <a
            href="https://github.com/hadronomy/ram"
            target="_blank"
            rel="noopener noreferrer"
            className="px-6 py-3 rounded-lg border border-fd-border bg-fd-background text-fd-foreground font-medium flex items-center gap-2 hover:bg-fd-accent transition-colors"
          >
            <Code className="w-4 h-4" /> GitHub
          </a>
        </div>
      </section>

      {/* Features Section */}
      <section className="w-full py-16 px-4 bg-fd-accent/30">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">Key Features</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div className="bg-fd-background p-6 rounded-xl border border-fd-border">
              <div className="w-12 h-12 rounded-lg bg-indigo-100 dark:bg-indigo-950 flex items-center justify-center mb-4">
                <Code className="w-6 h-6 text-indigo-500" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Formal Language</h3>
              <p className="text-fd-muted-foreground">
                A clear, consistent, and unambiguous specification for the RAM programming language.
              </p>
            </div>
            <div className="bg-fd-background p-6 rounded-xl border border-fd-border">
              <div className="w-12 h-12 rounded-lg bg-purple-100 dark:bg-purple-950 flex items-center justify-center mb-4">
                <Cpu className="w-6 h-6 text-purple-500" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Accurate Emulation</h3>
              <p className="text-fd-muted-foreground">
                An emulator that faithfully executes RAM programs according to the defined semantics.
              </p>
            </div>
            <div className="bg-fd-background p-6 rounded-xl border border-fd-border">
              <div className="w-12 h-12 rounded-lg bg-pink-100 dark:bg-pink-950 flex items-center justify-center mb-4">
                <Terminal className="w-6 h-6 text-pink-500" />
              </div>
              <h3 className="text-xl font-semibold mb-2">Performance</h3>
              <p className="text-fd-muted-foreground">
                Optimized for reasonable performance to handle non-trivial programs efficiently.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Documentation Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-4">Documentation</h2>
          <p className="text-fd-muted-foreground text-center max-w-2xl mx-auto mb-12">
            Comprehensive guides and references to help you learn and use the RAM language effectively.
          </p>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Link href="/docs" className="group">
              <div className="h-full p-6 rounded-xl border border-fd-border bg-fd-background hover:border-indigo-500/50 hover:shadow-md transition-all">
                <div className="flex items-center gap-3 mb-4">
                  <BookOpen className="w-5 h-5 text-indigo-500" />
                  <h3 className="text-xl font-semibold">Getting Started</h3>
                </div>
                <p className="text-fd-muted-foreground mb-4">
                  Learn the basics of RAM language, installation, and how to write your first program.
                </p>
                <span className="text-indigo-500 font-medium flex items-center gap-1 group-hover:gap-2 transition-all">
                  Read more <ArrowRight className="w-4 h-4" />
                </span>
              </div>
            </Link>
            <Link href="/docs" className="group">
              <div className="h-full p-6 rounded-xl border border-fd-border bg-fd-background hover:border-purple-500/50 hover:shadow-md transition-all">
                <div className="flex items-center gap-3 mb-4">
                  <Code className="w-5 h-5 text-purple-500" />
                  <h3 className="text-xl font-semibold">Language Reference</h3>
                </div>
                <p className="text-fd-muted-foreground mb-4">
                  Detailed documentation of RAM language syntax, instructions, and programming model.
                </p>
                <span className="text-purple-500 font-medium flex items-center gap-1 group-hover:gap-2 transition-all">
                  Read more <ArrowRight className="w-4 h-4" />
                </span>
              </div>
            </Link>
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="w-full py-16 px-4 bg-fd-accent/30">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-12">RAM Language Example</h2>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-center">
            <div>
              <h3 className="text-xl font-semibold mb-4">Simple Addition Program</h3>
              <p className="text-fd-muted-foreground mb-6">
                This example demonstrates a basic RAM program that adds two numbers stored in memory locations 1 and 2,
                then stores the result in location 3.
              </p>
              <Link
                href="/docs"
                className="px-5 py-2 rounded-lg bg-fd-background border border-fd-border text-fd-foreground font-medium flex items-center gap-2 hover:bg-fd-accent transition-colors w-fit"
              >
                Learn more about syntax <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
            <div className="bg-fd-background rounded-xl border border-fd-border p-4 overflow-hidden">
              <pre className="overflow-x-auto text-sm">
                <code className="language-ram">
                  <span className="text-gray-500"># Simple RAM program that adds two numbers</span>{'\n'}
                  <span className="text-indigo-500">LOAD</span> 1    <span className="text-gray-500"># Load value from address 1</span>{'\n'}
                  <span className="text-indigo-500">ADD</span> 2     <span className="text-gray-500"># Add value from address 2</span>{'\n'}
                  <span className="text-indigo-500">STORE</span> 3   <span className="text-gray-500"># Store result in address 3</span>{'\n'}
                  <span className="text-indigo-500">HALT</span>      <span className="text-gray-500"># Stop execution</span>
                </code>
              </pre>
            </div>
          </div>
        </div>
      </section>

      {/* Getting Started Section */}
      <section className="w-full py-16 px-4">
        <div className="max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-4">Getting Started</h2>
          <p className="text-fd-muted-foreground text-center max-w-2xl mx-auto mb-12">
            Follow these steps to install and start using RAM on your system.
          </p>
          <div className="bg-fd-background rounded-xl border border-fd-border p-6">
            <h3 className="text-xl font-semibold mb-4">Installation</h3>
            <p className="text-fd-muted-foreground mb-6">
              RAM is available as an npm package. You can install it globally using npm, yarn, or pnpm:
            </p>
            <div className="space-y-6">
              <div className="bg-fd-accent/50 rounded-lg p-4">
                <h4 className="font-medium mb-2">Using npm</h4>
                <pre className="bg-fd-background p-3 rounded-md overflow-x-auto">
                  <code>
                    npm install -g @ramlang/cli
                  </code>
                </pre>
              </div>
              <div className="bg-fd-accent/50 rounded-lg p-4">
                <h4 className="font-medium mb-2">Using yarn</h4>
                <pre className="bg-fd-background p-3 rounded-md overflow-x-auto">
                  <code>
                    yarn global add @ramlang/cli
                  </code>
                </pre>
              </div>
              <div className="bg-fd-accent/50 rounded-lg p-4">
                <h4 className="font-medium mb-2">Using pnpm</h4>
                <pre className="bg-fd-background p-3 rounded-md overflow-x-auto">
                  <code>
                    pnpm add -g @ramlang/cli
                  </code>
                </pre>
              </div>
            </div>

            <h3 className="text-xl font-semibold mt-8 mb-4">Usage</h3>
            <p className="text-fd-muted-foreground mb-6">
              After installation, you can use the <code className="bg-fd-accent/30 px-1.5 py-0.5 rounded">ram</code> command to run RAM programs:
            </p>
            <div className="bg-fd-accent/50 rounded-lg p-4 mb-6">
              <h4 className="font-medium mb-2">Running a RAM program</h4>
              <pre className="bg-fd-background p-3 rounded-md overflow-x-auto">
                <code>
                  ram run path/to/your/program.ram
                </code>
              </pre>
            </div>
            <Link
              href="/docs"
              className="px-5 py-2 rounded-lg bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-medium flex items-center gap-2 hover:opacity-90 transition-opacity w-fit"
            >
              View full documentation <ArrowRight className="w-4 h-4" />
            </Link>
          </div>
        </div>
      </section>

      {/* Footer CTA */}
      <section className="w-full py-16 px-4 text-center">
        <h2 className="text-3xl font-bold mb-4">Ready to get started?</h2>
        <p className="text-fd-muted-foreground max-w-2xl mx-auto mb-8">
          Dive into the documentation to learn more about RAM language and start building your own programs.
        </p>
        <Link
          href="/docs"
          className="px-6 py-3 rounded-lg bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-medium inline-flex items-center gap-2 hover:opacity-90 transition-opacity"
        >
          Explore the docs <ArrowRight className="w-4 h-4" />
        </Link>
      </section>
    </main>
  );
}

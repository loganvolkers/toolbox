export type CliArg = {
  name: string;
  long: string | null;
  positional: boolean;
  described: boolean;
  help?: string | null;
};

export type CliCommand = {
  name: string;
  slug: string;
  about: string | null;
  args: CliArg[];
  mcpTool: string | null;
  structuredOutput: boolean;
  usage?: string | null;
  helpText?: string | null;
};

export type McpTool = {
  name: string;
  description: string | null;
  cliCommands: string[];
  inputSchema: {
    properties: Record<string, { type?: string; enum?: string[]; description?: string }>;
    required: string[];
  };
};

export type CliCatalog = {
  commands: CliCommand[];
  mcpTools: McpTool[];
};

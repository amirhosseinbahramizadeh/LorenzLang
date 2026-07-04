import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    const runFileCommand = vscode.commands.registerCommand('lorenz.runFile', () => {
        const editor = vscode.window.activeTextEditor;

        if (!editor) {
            vscode.window.showWarningMessage('No active editor found.');
            return;
        }

        const document = editor.document;
        const filePath = document.fileName;

        if (!filePath.endsWith('.lz')) {
            vscode.window.showWarningMessage('Lorenz Runner only works on .lz files.');
            return;
        }

        const terminal = vscode.window.createTerminal({
            name: 'Lorenz',
            cwd: require('path').dirname(filePath)
        });

        terminal.show();
        terminal.sendText(`lorenz run "${filePath}"`);
    });

    context.subscriptions.push(runFileCommand);
}

export function deactivate() {}
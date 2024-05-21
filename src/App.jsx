import React, { useState } from 'react';
import { List, Button, Flex } from 'antd';
import { CaretRightOutlined } from '@ant-design/icons';
import { open } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api/tauri';

function App() {
    const [directories, setDirectories] = useState([]);

    const selectFolder = async () => {
        try {
            // 使用 'open' 函数从用户那里获取文件夹路径
            const selected = await open({
                directory: true, // 设置为 true 以选择文件夹而非文件
                multiple: false, // 可以设置为 true 如果你想允许选择多个文件夹
            });

            // 'selected' 可能是一个包含一个或多个路径的 string 或者 string 数组
            console.log(selected);
            return selected;
        } catch (error) {
            console.error('Failed to select folder:', error);
        }
    }

    const listDirectories = async (path) => {
        try {
            const dirs = await invoke('read_directory', { path });
            setDirectories(dirs);
        } catch (error) {
            console.error('Error reading directory:', error);
        }
    };

    const readPackageJson = async (path) => {
        try {
            const packageJson = await invoke('read_package_json_files', { path });
            // Add logic to parse JSON and execute scripts
            console.log('packageJson', packageJson)
            return packageJson;
        } catch (error) {
            console.error('Error reading package.json:', error);
        }
    };

    const runCommand = async (path, command) => {
        try {
            const output = await invoke('exec_command', {
                path: path,
                command: command
            });
            console.log('Command output:', output);
        } catch (error) {
            console.error('Command failed:', error);
        }
    }

    // Call listDirectories with the path you want to list on component mount

    return (
        <div style={{ padding: '12px 16px' }}>
            <Flex gap={8} style={{ marginBottom: '12px' }}>
                <Button type={'primary'}
                        onClick={async () => {
                            const path = await selectFolder();
                            if (path) {
                                const result = await readPackageJson(path)
                                setDirectories(result);
                            }
                        }}>
                    选择文件夹
                </Button>
                <Button onClick={() => {
                    setDirectories([])
                }}>
                    清空
                </Button>
            </Flex>
            <List
                size="large"
                bordered
                dataSource={directories}
                renderItem={item => {
                    console.log('item', item);

                    // TODO: 可以改为scripts命令列表
                    return (
                        <List.Item>
                            <List.Item.Meta title={item.name} description={item.description || '暂无描述'}/>
                            <Button icon={<CaretRightOutlined style={{ fontSize: '20px', color: '#389e0d' }}/>}
                                    onClick={() => {
                                        console.log('haha', item.path, item.scripts.start)
                                        runCommand(item.path, item.scripts.start)
                                    }}/>
                        </List.Item>
                    )
                }}
            />
        </div>
    );
}

export default App;

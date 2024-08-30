import {Button, Flex, Modal, Progress, Row, Typography} from "antd";
import {useEffect, useState} from "react";
import {async_run} from "../tool/ReactTool.ts";
import {check} from "@tauri-apps/plugin-updater";
import {relaunch} from "@tauri-apps/plugin-process";
import {LoadingOutlined} from "@ant-design/icons";

function UpdateModal() {
    const [isModalVisible, setIsModalVisible] = useState<boolean>(false);
    const [progress, setProgress] = useState<number>(0); // 进度条的当前值
    const [updating, setUpdating] = useState<boolean>(false); // 是否正在更新
    const [content, setContent] = useState({
        version: "",
        notes: "",
        date: ""
    })

    // 检查更新
    useEffect(() => {
        async_run(async () => {
            const update = await check();
            if (update) {
                setContent({
                    version: update.version,
                    notes: update.body ? update.body : "",
                    date: update.date ? update.date : ""
                })
                showModal()
            }
        })
    }, [])

    const showModal = (): void => {
        setIsModalVisible(true);
    };

    const handleOk = async () => {
        setUpdating(true);
        // 更新
        const update = await check();
        if (update) {
            let downloaded = 0;
            let contentLength = 0;
            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength as number;
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        setProgress(parseFloat(((downloaded / contentLength) * 100).toFixed(1)))
                        break;
                    case 'Finished':
                        break;
                }
            });
            await relaunch();
        }
        handleCancel()
    };

    // 模拟更新过程
    // function simulateUpdate(): void {
    //     setUpdating(true)
    //     let currentProgress = 0;
    //     const interval = setInterval(() => {
    //         if (currentProgress < 100) {
    //             currentProgress += 0.78979846518978; // 更新进度
    //             setProgress(Math.floor((currentProgress / 100) * 1000000));
    //         } else {
    //             clearInterval(interval);
    //             setUpdating(false);
    //             // 更新完成后的逻辑
    //         }
    //     }, 50); // 每隔500ms更新一次进度
    // }

    const handleCancel = (): void => {
        if (!updating) {
            setIsModalVisible(false);
            setUpdating(false);
            setProgress(0);
        }
    };

    return (
        <Modal
            title="检测到新版本"
            open={isModalVisible}
            onCancel={handleCancel}
            footer={null} // 取消默认的确定和取消按钮
        >
            <Row>
                <Typography.Title level={5}>新版本:{content.version}</Typography.Title>
            </Row>
            <Row>
                <Typography.Text>日期:{content.date}</Typography.Text>
            </Row>
            <Row>
                <Typography.Text>日志:{content.notes}</Typography.Text>
            </Row>
            <Row justify={"center"}>
                {updating ? (
                    <Flex align={"center"} vertical={true} style={{width: '100%'}}>
                        <Progress percent={progress}/>
                        <Typography.Text>程序会自动重启，请耐心等待</Typography.Text>
                        <LoadingOutlined/>
                    </Flex>
                ) : (
                    <Button type="primary" onClick={handleOk}>
                        更新
                    </Button>
                )}
            </Row>
        </Modal>
    )
}

export default UpdateModal
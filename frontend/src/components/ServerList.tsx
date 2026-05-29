import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Layout, Table, Button, Modal, Form, Input, Select, InputNumber,
  Typography, Tag, message, Popconfirm, Space,
} from 'antd';
import { PlusOutlined, DeleteOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import type { MonitorServer, ServiceInstance, CreateServerRequest, CreateInstanceRequest } from '../types';
import { listServers, createServer, deleteServer, listInstances, registerInstance, deleteInstance } from '../api/client';

const { Header, Content } = Layout;
const { Title } = Typography;

const typeColorMap: Record<string, string> = {
  'dotnet-ocr': 'blue',
  'cpp-ocr': 'green',
  'rust-ocr': 'orange',
};

const ServerList: React.FC = () => {
  const navigate = useNavigate();
  const [servers, setServers] = useState<MonitorServer[]>([]);
  const [instancesMap, setInstancesMap] = useState<Record<number, ServiceInstance[]>>({});
  const [loading, setLoading] = useState(true);
  const [serverModalOpen, setServerModalOpen] = useState(false);
  const [instanceModalOpen, setInstanceModalOpen] = useState(false);
  const [selectedServerId, setSelectedServerId] = useState<number | null>(null);
  const [serverForm] = Form.useForm();
  const [instanceForm] = Form.useForm();

  const fetchData = async () => {
    setLoading(true);
    try {
      const serverList = await listServers();
      setServers(serverList);
      const map: Record<number, ServiceInstance[]> = {};
      for (const s of serverList) {
        try {
          map[s.id] = await listInstances(s.id);
        } catch {
          map[s.id] = [];
        }
      }
      setInstancesMap(map);
    } catch {
      message.error('Failed to load servers');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  const handleCreateServer = async (values: CreateServerRequest) => {
    try {
      await createServer(values);
      message.success('Server created');
      setServerModalOpen(false);
      serverForm.resetFields();
      fetchData();
    } catch {
      message.error('Failed to create server');
    }
  };

  const handleDeleteServer = async (id: number) => {
    try {
      await deleteServer(id);
      message.success('Server deleted');
      fetchData();
    } catch {
      message.error('Failed to delete server');
    }
  };

  const handleRegisterInstance = async (values: CreateInstanceRequest) => {
    if (!selectedServerId) return;
    try {
      await registerInstance(selectedServerId, values);
      message.success('Instance registered');
      setInstanceModalOpen(false);
      instanceForm.resetFields();
      fetchData();
    } catch {
      message.error('Failed to register instance');
    }
  };

  const handleDeleteInstance = async (id: number) => {
    try {
      await deleteInstance(id);
      message.success('Instance deleted');
      fetchData();
    } catch {
      message.error('Failed to delete instance');
    }
  };

  const serverColumns = [
    {
      title: 'ID',
      dataIndex: 'id',
      key: 'id',
      width: 60,
    },
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      render: (text: string, record: MonitorServer) => (
        <a onClick={() => navigate(`/servers/${record.id}`)}>{text}</a>
      ),
    },
    {
      title: 'Description',
      dataIndex: 'description',
      key: 'description',
    },
    {
      title: 'Instances',
      key: 'instance_count',
      render: (_: any, record: MonitorServer) => instancesMap[record.id]?.length ?? 0,
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: MonitorServer) => (
        <Space>
          <Button
            type="primary"
            size="small"
            icon={<PlusOutlined />}
            onClick={() => { setSelectedServerId(record.id); setInstanceModalOpen(true); }}
          >
            Add Instance
          </Button>
          <Popconfirm title="Delete this server and all its instances?" onConfirm={() => handleDeleteServer(record.id)}>
            <Button danger size="small" icon={<DeleteOutlined />}>Delete</Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0' }}>
        <Space>
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/')} />
          <Title level={3} style={{ margin: '16px 0' }}>Server Management</Title>
        </Space>
      </Header>
      <Content style={{ padding: '24px' }}>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setServerModalOpen(true)}
          style={{ marginBottom: 16 }}
        >
          Create Server Group
        </Button>

        <Table
          rowKey="id"
          columns={serverColumns}
          dataSource={servers}
          loading={loading}
          pagination={false}
          expandable={{
            expandedRowRender: (record: MonitorServer) => {
              const instances = instancesMap[record.id] || [];
              if (instances.length === 0) return <p style={{ color: '#999' }}>No instances</p>;
              return (
                <Table
                  rowKey="id"
                  size="small"
                  pagination={false}
                  dataSource={instances}
                  columns={[
                    { title: 'ID', dataIndex: 'id', width: 60 },
                    {
                      title: 'Name',
                      dataIndex: 'name',
                      render: (text: string, inst: ServiceInstance) => (
                        <a onClick={() => navigate(`/instances/${inst.id}`)}>{text}</a>
                      ),
                    },
                    {
                      title: 'Type',
                      dataIndex: 'service_type',
                      render: (type: string) => <Tag color={typeColorMap[type] || 'default'}>{type}</Tag>,
                    },
                    { title: 'Base URL', dataIndex: 'base_url' },
                    { title: 'Poll Interval', dataIndex: 'poll_interval_secs', render: (v: number) => `${v}s` },
                    {
                      title: 'Actions',
                      render: (_: any, inst: ServiceInstance) => (
                        <Popconfirm title="Delete this instance?" onConfirm={() => handleDeleteInstance(inst.id)}>
                          <Button danger size="small" icon={<DeleteOutlined />}>Delete</Button>
                        </Popconfirm>
                      ),
                    },
                  ]}
                />
              );
            },
          }}
        />

        <Modal
          title="Create Server Group"
          open={serverModalOpen}
          onCancel={() => setServerModalOpen(false)}
          onOk={() => serverForm.submit()}
        >
          <Form form={serverForm} onFinish={handleCreateServer} layout="vertical">
            <Form.Item name="name" label="Name" rules={[{ required: true }]}>
              <Input placeholder="e.g. Production OCR Cluster" />
            </Form.Item>
            <Form.Item name="description" label="Description">
              <Input placeholder="Optional description" />
            </Form.Item>
          </Form>
        </Modal>

        <Modal
          title="Register Instance"
          open={instanceModalOpen}
          onCancel={() => setInstanceModalOpen(false)}
          onOk={() => instanceForm.submit()}
        >
          <Form form={instanceForm} onFinish={handleRegisterInstance} layout="vertical">
            <Form.Item name="name" label="Name" rules={[{ required: true }]}>
              <Input placeholder="e.g. OCR Server #1" />
            </Form.Item>
            <Form.Item name="service_type" label="Type" rules={[{ required: true }]}>
              <Select placeholder="Select type">
                <Select.Option value="dotnet-ocr">.NET OCR</Select.Option>
                <Select.Option value="cpp-ocr">C++ OCR</Select.Option>
                <Select.Option value="rust-ocr">Rust OCR</Select.Option>
              </Select>
            </Form.Item>
            <Form.Item name="base_url" label="Base URL" rules={[{ required: true }]}>
              <Input placeholder="e.g. http://192.168.1.10:21600" />
            </Form.Item>
            <Form.Item name="poll_interval_secs" label="Poll Interval (s)" initialValue={10}>
              <InputNumber min={1} max={300} />
            </Form.Item>
          </Form>
        </Modal>
      </Content>
    </Layout>
  );
};

export default ServerList;

import React, { useEffect, useState } from 'react';
import {
  Layout, Table, Button, Modal, Form, Input, Select, InputNumber,
  Typography, Tag, message, Popconfirm, Space,
} from 'antd';
import { PlusOutlined, DeleteOutlined, ArrowLeftOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { Service, CreateServiceRequest } from '../types';
import { listServices, registerService, deleteService } from '../api/client';

const { Header, Content } = Layout;
const { Title } = Typography;

const ServiceList: React.FC = () => {
  const navigate = useNavigate();
  const [services, setServices] = useState<Service[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [form] = Form.useForm();

  const fetchServices = async () => {
    setLoading(true);
    try {
      const data = await listServices();
      setServices(data);
    } catch {
      message.error('Failed to load services');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchServices(); }, []);

  const handleRegister = async (values: CreateServiceRequest) => {
    try {
      await registerService(values);
      message.success('Service registered');
      setModalOpen(false);
      form.resetFields();
      fetchServices();
    } catch {
      message.error('Failed to register service');
    }
  };

  const handleDelete = async (id: number) => {
    try {
      await deleteService(id);
      message.success('Service deleted');
      fetchServices();
    } catch {
      message.error('Failed to delete service');
    }
  };

  const columns = [
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
      render: (text: string, record: Service) => (
        <a onClick={() => navigate(`/services/${record.id}`)}>{text}</a>
      ),
    },
    {
      title: 'Type',
      dataIndex: 'service_type',
      key: 'service_type',
      render: (type: string) => {
        const colorMap: Record<string, string> = {
          'dotnet-ocr': 'blue',
          'cpp-ocr': 'green',
          'rust-ocr': 'orange',
        };
        return <Tag color={colorMap[type] || 'default'}>{type}</Tag>;
      },
    },
    {
      title: 'Base URL',
      dataIndex: 'base_url',
      key: 'base_url',
    },
    {
      title: 'Poll Interval',
      dataIndex: 'poll_interval_secs',
      key: 'poll_interval_secs',
      render: (v: number) => `${v}s`,
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_: any, record: Service) => (
        <Space>
          <Popconfirm
            title="Delete this service?"
            onConfirm={() => handleDelete(record.id)}
          >
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
          <Title level={3} style={{ margin: '16px 0' }}>Service Management</Title>
        </Space>
      </Header>
      <Content style={{ padding: '24px' }}>
        <Button
          type="primary"
          icon={<PlusOutlined />}
          onClick={() => setModalOpen(true)}
          style={{ marginBottom: 16 }}
        >
          Register Service
        </Button>

        <Table
          rowKey="id"
          columns={columns}
          dataSource={services}
          loading={loading}
          pagination={false}
        />

        <Modal
          title="Register New Service"
          open={modalOpen}
          onCancel={() => setModalOpen(false)}
          onOk={() => form.submit()}
        >
          <Form form={form} onFinish={handleRegister} layout="vertical">
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

export default ServiceList;

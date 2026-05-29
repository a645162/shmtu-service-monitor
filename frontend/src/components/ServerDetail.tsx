import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Layout, Card, Descriptions, Typography, Spin, Alert, Row, Col, Tag, Badge,
  Button, Space,
} from 'antd';
import { ArrowLeftOutlined, CheckCircleOutlined, WarningOutlined, CloseCircleOutlined } from '@ant-design/icons';
import type { ServerDetail, InstanceDashboardEntry } from '../types';
import { getServerDetail } from '../api/client';

const { Header, Content } = Layout;
const { Title } = Typography;

const statusConfig: Record<string, { color: string; icon: React.ReactNode }> = {
  healthy: { color: 'success', icon: <CheckCircleOutlined /> },
  busy: { color: 'warning', icon: <WarningOutlined /> },
  unavailable: { color: 'error', icon: <CloseCircleOutlined /> },
};

const typeColorMap: Record<string, string> = {
  'dotnet-ocr': 'blue',
  'cpp-ocr': 'green',
  'rust-ocr': 'orange',
};

const ServerDetailView: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [data, setData] = useState<ServerDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const serverId = parseInt(id || '0', 10);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      setError(null);
      try {
        const res = await getServerDetail(serverId);
        setData(res);
      } catch (e: any) {
        setError(e.message);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [serverId]);

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '100px auto' }} />;
  if (error) return <Alert type="error" message={error} />;
  if (!data) return <Alert type="error" message="Server not found" />;

  const healthyCount = data.instances.filter(e => e.latest_status?.status === 'healthy').length;

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0' }}>
        <Space>
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/servers')} />
          <Title level={3} style={{ margin: '16px 0' }}>{data.server.name}</Title>
        </Space>
      </Header>
      <Content style={{ padding: '24px' }}>
        <Card title="Server Info" style={{ marginBottom: 16 }}>
          <Descriptions column={2} bordered>
            <Descriptions.Item label="ID">{data.server.id}</Descriptions.Item>
            <Descriptions.Item label="Name">{data.server.name}</Descriptions.Item>
            <Descriptions.Item label="Description" span={2}>{data.server.description || '—'}</Descriptions.Item>
            <Descriptions.Item label="Instances">{data.instances.length}</Descriptions.Item>
            <Descriptions.Item label="Healthy">{healthyCount}/{data.instances.length}</Descriptions.Item>
          </Descriptions>
        </Card>

        <Row gutter={[16, 16]}>
          {data.instances.map((entry: InstanceDashboardEntry) => {
            const inst = entry.instance;
            const st = entry.latest_status;
            const cfg = st ? statusConfig[st.status] || statusConfig.unavailable : statusConfig.unavailable;

            return (
              <Col xs={24} sm={12} lg={8} key={inst.id}>
                <Card
                  hoverable
                  onClick={() => navigate(`/instances/${inst.id}`)}
                  title={
                    <span>
                      <Badge status={cfg.color as any} />
                      {inst.name}
                    </span>
                  }
                  extra={<Tag color={typeColorMap[inst.service_type] || 'default'}>{inst.service_type}</Tag>}
                >
                  {st ? (
                    <>
                      <p><strong>Status:</strong> <Tag icon={cfg.icon} color={cfg.color}>{st.status}</Tag></p>
                      <p><strong>Models:</strong> {st.models_loaded ? 'Loaded' : 'Not Loaded'}</p>
                      <p><strong>Queue:</strong> {st.pending_requests}/{st.queue_capacity}</p>
                      <p><strong>Response:</strong> {st.response_time_ms.toFixed(1)}ms</p>
                    </>
                  ) : (
                    <p style={{ color: '#999' }}>No status data</p>
                  )}
                </Card>
              </Col>
            );
          })}
        </Row>
      </Content>
    </Layout>
  );
};

export default ServerDetailView;

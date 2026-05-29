import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Layout, Row, Col, Card, Statistic, Tag, Badge, Typography, Spin, Alert,
} from 'antd';
import {
  CheckCircleOutlined, WarningOutlined, CloseCircleOutlined,
  DashboardOutlined, CloudServerOutlined, ReloadOutlined,
} from '@ant-design/icons';
import type { DashboardSummary, ServerDashboardEntry, InstanceDashboardEntry } from '../types';
import { getDashboard } from '../api/client';

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

const Dashboard: React.FC = () => {
  const navigate = useNavigate();
  const [data, setData] = useState<DashboardSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getDashboard();
      setData(res);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { fetchData(); }, []);

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '100px auto' }} />;
  if (error) return <Alert type="error" message={error} />;
  if (!data) return null;

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0' }}>
        <Title level={3} style={{ margin: '16px 0' }}>
          <DashboardOutlined /> SHMTU Service Monitor
        </Title>
      </Header>
      <Content style={{ padding: '24px' }}>
        <Row gutter={[16, 16]} style={{ marginBottom: 24 }}>
          <Col xs={12} sm={6}>
            <Card>
              <Statistic title="Server Groups" value={data.total_servers} prefix={<CloudServerOutlined />} />
            </Card>
          </Col>
          <Col xs={12} sm={6}>
            <Card>
              <Statistic
                title="Instances"
                value={data.total_instances}
                prefix={<CloudServerOutlined />}
              />
            </Card>
          </Col>
          <Col xs={12} sm={6}>
            <Card>
              <Statistic
                title="Healthy"
                value={data.healthy_instances}
                valueStyle={{ color: '#52c41a' }}
                prefix={<CheckCircleOutlined />}
              />
            </Card>
          </Col>
          <Col xs={12} sm={6}>
            <Card>
              <Statistic
                title="Unavailable"
                value={data.unavailable_instances}
                valueStyle={{ color: '#ff4d4f' }}
                prefix={<CloseCircleOutlined />}
              />
            </Card>
          </Col>
        </Row>

        <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
          <Col>
            <a onClick={fetchData} style={{ cursor: 'pointer' }}><ReloadOutlined /> Refresh</a>
          </Col>
          <Col>
            <a onClick={() => navigate('/servers')} style={{ cursor: 'pointer' }}><CloudServerOutlined /> Manage Servers</a>
          </Col>
        </Row>

        {data.servers.map((serverEntry: ServerDashboardEntry) => {
          const serverHealthy = serverEntry.instances.filter(
            e => e.latest_status?.status === 'healthy'
          ).length;
          const serverTotal = serverEntry.instances.length;

          return (
            <Card
              key={serverEntry.server.id}
              title={
                <span>
                  <Badge
                    status={serverHealthy === serverTotal ? 'success' : serverHealthy > 0 ? 'warning' : 'error'}
                  />
                  {serverEntry.server.name}
                  <span style={{ color: '#999', fontWeight: 'normal', marginLeft: 8, fontSize: 14 }}>
                    {serverHealthy}/{serverTotal} healthy
                  </span>
                </span>
              }
              extra={
                <a onClick={() => navigate(`/servers/${serverEntry.server.id}`)}>Details</a>
              }
              style={{ marginBottom: 16 }}
            >
              {serverEntry.server.description && (
                <p style={{ color: '#666', marginBottom: 12 }}>{serverEntry.server.description}</p>
              )}
              <Row gutter={[16, 16]}>
                {serverEntry.instances.map((entry: InstanceDashboardEntry) => {
                  const inst = entry.instance;
                  const st = entry.latest_status;
                  const cfg = st ? statusConfig[st.status] || statusConfig.unavailable : statusConfig.unavailable;

                  return (
                    <Col xs={24} sm={12} lg={8} xl={6} key={inst.id}>
                      <Card
                        hoverable
                        size="small"
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
            </Card>
          );
        })}
      </Content>
    </Layout>
  );
};

export default Dashboard;

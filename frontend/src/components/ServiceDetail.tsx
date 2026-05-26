import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  Layout, Card, Descriptions, Tag, Typography, Spin, Alert, Row, Col, Statistic, Progress,
  Button, Space,
} from 'antd';
import {
  ArrowLeftOutlined, CheckCircleOutlined, WarningOutlined, CloseCircleOutlined,
} from '@ant-design/icons';
import type { Service, ServiceStatus } from '../types';
import { getService, getServiceStatus, getServiceHistory } from '../api/client';
import StatusTimeline from './StatusTimeline';

const { Header, Content } = Layout;
const { Title } = Typography;

const statusIconMap: Record<string, React.ReactNode> = {
  healthy: <CheckCircleOutlined />,
  busy: <WarningOutlined />,
  unavailable: <CloseCircleOutlined />,
};

const ServiceDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [service, setService] = useState<Service | null>(null);
  const [status, setStatus] = useState<ServiceStatus | null>(null);
  const [history, setHistory] = useState<ServiceStatus[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const serviceId = parseInt(id || '0', 10);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);
      setError(null);
      try {
        const [svc, st, hist] = await Promise.all([
          getService(serviceId),
          getServiceStatus(serviceId).catch(() => null),
          getServiceHistory(serviceId, { limit: 200 }).catch(() => []),
        ]);
        setService(svc);
        setStatus(st);
        setHistory(hist);
      } catch (e: any) {
        setError(e.message);
      } finally {
        setLoading(false);
      }
    };
    fetchData();
  }, [serviceId]);

  if (loading) return <Spin size="large" style={{ display: 'block', margin: '100px auto' }} />;
  if (error) return <Alert type="error" message={error} />;
  if (!service) return <Alert type="error" message="Service not found" />;

  const tagColor = status?.status === 'healthy' ? 'success' : status?.status === 'busy' ? 'warning' : 'error';

  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Header style={{ background: '#fff', padding: '0 24px', borderBottom: '1px solid #f0f0f0' }}>
        <Space>
          <Button icon={<ArrowLeftOutlined />} onClick={() => navigate('/services')} />
          <Title level={3} style={{ margin: '16px 0' }}>{service.name}</Title>
        </Space>
      </Header>
      <Content style={{ padding: '24px' }}>
        <Row gutter={[16, 16]}>
          <Col xs={24} lg={12}>
            <Card title="Service Info">
              <Descriptions column={1} bordered>
                <Descriptions.Item label="ID">{service.id}</Descriptions.Item>
                <Descriptions.Item label="Name">{service.name}</Descriptions.Item>
                <Descriptions.Item label="Type">
                  <Tag color={service.service_type === 'dotnet-ocr' ? 'blue' : service.service_type === 'cpp-ocr' ? 'green' : 'orange'}>
                    {service.service_type}
                  </Tag>
                </Descriptions.Item>
                <Descriptions.Item label="Base URL">{service.base_url}</Descriptions.Item>
                <Descriptions.Item label="Poll Interval">{service.poll_interval_secs}s</Descriptions.Item>
                <Descriptions.Item label="Status">
                  {status ? (
                    <Tag icon={statusIconMap[status.status]} color={tagColor}>{status.status}</Tag>
                  ) : 'No data'}
                </Descriptions.Item>
              </Descriptions>
            </Card>
          </Col>

          <Col xs={24} lg={12}>
            <Card title="Current Status">
              {status ? (
                <Row gutter={[16, 16]}>
                  <Col span={8}>
                    <Statistic title="Response Time" value={status.response_time_ms.toFixed(1)} suffix="ms" />
                  </Col>
                  <Col span={8}>
                    <Statistic title="Queue Usage" value={`${status.pending_requests}/${status.queue_capacity}`} />
                  </Col>
                  <Col span={8}>
                    <Progress
                      type="circle"
                      percent={Math.round(status.utilization_percent)}
                      format={(p) => `${p}%`}
                      strokeColor={status.utilization_percent > 80 ? '#ff4d4f' : status.utilization_percent > 50 ? '#faad14' : '#52c41a'}
                    />
                  </Col>
                  <Col span={12}>
                    <Statistic title="Models Loaded" value={status.models_loaded ? 'Yes' : 'No'} />
                  </Col>
                  <Col span={12}>
                    <Statistic title="Avg Response" value={status.avg_response_ms?.toFixed(1) ?? 'N/A'} suffix="ms" />
                  </Col>
                  {status.total_requests != null && (
                    <>
                      <Col span={8}>
                        <Statistic title="Total Requests" value={status.total_requests} />
                      </Col>
                      <Col span={8}>
                        <Statistic title="Success" value={status.success_count ?? 0} valueStyle={{ color: '#52c41a' }} />
                      </Col>
                      <Col span={8}>
                        <Statistic title="Failure" value={status.failure_count ?? 0} valueStyle={{ color: '#ff4d4f' }} />
                      </Col>
                    </>
                  )}
                </Row>
              ) : (
                <p style={{ color: '#999' }}>No status data available</p>
              )}
            </Card>
          </Col>
        </Row>

        <Card title="Status Timeline" style={{ marginTop: 16 }}>
          <StatusTimeline history={history} />
        </Card>
      </Content>
    </Layout>
  );
};

export default ServiceDetail;

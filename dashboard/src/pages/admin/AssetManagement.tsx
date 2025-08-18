import React, { useState } from 'react';
import {
  Box,
  Heading,
  VStack,
  HStack,
  Text,
  Button,
  Input,
  Badge,
  SimpleGrid,
} from '@chakra-ui/react';

interface Asset {
  id: string;
  name: string;
  type: 'Server' | 'Workstation' | 'Laptop' | 'Mobile' | 'IoT Device' | 'Network Equipment';
  ipAddress: string;
  macAddress: string;
  operatingSystem: string;
  department: string;
  assignedTo: string;
  location: string;
  status: 'Online' | 'Offline' | 'Maintenance' | 'Decommissioned';
  lastSeen: string;
  securityLevel: 'High' | 'Medium' | 'Low';
  agentInstalled: boolean;
  vulnerabilities: number;
  complianceScore: number;
}

const AssetManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedType, setSelectedType] = useState('all');
  const [selectedStatus, setSelectedStatus] = useState('all');
  const [showAddAsset, setShowAddAsset] = useState(false);
  const [newAsset, setNewAsset] = useState({
    name: '',
    type: 'Workstation' as const,
    ipAddress: '',
    department: '',
    assignedTo: '',
    location: '',
  });

  // Mock data - would come from API
  const [assets] = useState<Asset[]>([
    {
      id: '1',
      name: 'WS-001-FINANCE',
      type: 'Workstation',
      ipAddress: '192.168.1.101',
      macAddress: '00:1A:2B:3C:4D:5E',
      operatingSystem: 'Windows 11 Pro',
      department: 'Finance',
      assignedTo: 'John Smith',
      location: 'Building A - Floor 2',
      status: 'Online',
      lastSeen: '2024-01-20 14:32',
      securityLevel: 'High',
      agentInstalled: true,
      vulnerabilities: 2,
      complianceScore: 92,
    },
    {
      id: '2',
      name: 'SRV-001-DATABASE',
      type: 'Server',
      ipAddress: '192.168.1.50',
      macAddress: '00:1A:2B:3C:4D:5F',
      operatingSystem: 'Ubuntu Server 22.04',
      department: 'IT',
      assignedTo: 'IT Team',
      location: 'Data Center - Rack 5',
      status: 'Online',
      lastSeen: '2024-01-20 14:35',
      securityLevel: 'High',
      agentInstalled: true,
      vulnerabilities: 0,
      complianceScore: 98,
    },
    {
      id: '3',
      name: 'LT-005-SALES',
      type: 'Laptop',
      ipAddress: '192.168.1.205',
      macAddress: '00:1A:2B:3C:4D:60',
      operatingSystem: 'Windows 11 Pro',
      department: 'Sales',
      assignedTo: 'Sarah Johnson',
      location: 'Remote',
      status: 'Offline',
      lastSeen: '2024-01-19 18:45',
      securityLevel: 'Medium',
      agentInstalled: false,
      vulnerabilities: 5,
      complianceScore: 74,
    },
  ]);

  const assetTypes = ['Server', 'Workstation', 'Laptop', 'Mobile', 'IoT Device', 'Network Equipment'];
  const statuses = ['Online', 'Offline', 'Maintenance', 'Decommissioned'];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Online': return 'green';
      case 'Offline': return 'red';
      case 'Maintenance': return 'orange';
      case 'Decommissioned': return 'gray';
      default: return 'gray';
    }
  };

  const getSecurityLevelColor = (level: string) => {
    switch (level) {
      case 'High': return 'green';
      case 'Medium': return 'orange';
      case 'Low': return 'red';
      default: return 'gray';
    }
  };

  const getComplianceColor = (score: number) => {
    if (score >= 90) return 'green';
    if (score >= 70) return 'orange';
    return 'red';
  };

  const filteredAssets = assets.filter(asset => {
    const matchesSearch = asset.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.assignedTo.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.department.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         asset.ipAddress.includes(searchTerm);
    const matchesType = selectedType === 'all' || asset.type === selectedType;
    const matchesStatus = selectedStatus === 'all' || asset.status === selectedStatus;
    return matchesSearch && matchesType && matchesStatus;
  });

  const handleAddAsset = () => {
    console.log('Adding asset:', newAsset);
    setShowAddAsset(false);
    setNewAsset({
      name: '',
      type: 'Workstation',
      ipAddress: '',
      department: '',
      assignedTo: '',
      location: '',
    });
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">Asset Management</Heading>
          <Text color="gray.600" mt={1}>
            Monitor and manage all IT assets across your organization
          </Text>
        </Box>
        <Button colorScheme="blue" onClick={() => setShowAddAsset(true)}>
          Add New Asset
        </Button>
      </HStack>

      {/* Statistics */}
      <SimpleGrid columns={{ base: 2, md: 5 }} gap={4}>
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="blue.600">
            {assets.length}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Assets</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {assets.filter(a => a.status === 'Online').length}
          </Text>
          <Text fontSize="sm" color="gray.600">Online</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="red.600">
            {assets.filter(a => a.vulnerabilities > 0).length}
          </Text>
          <Text fontSize="sm" color="gray.600">With Vulnerabilities</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {assets.filter(a => !a.agentInstalled).length}
          </Text>
          <Text fontSize="sm" color="gray.600">No Agent</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="purple.600">
            {Math.round(assets.reduce((sum, asset) => sum + asset.complianceScore, 0) / assets.length)}%
          </Text>
          <Text fontSize="sm" color="gray.600">Avg Compliance</Text>
        </Box>
      </SimpleGrid>

      {/* Filters */}
      <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack gap={4}>
          <Box w="100%">
            <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
              Search Assets
            </Text>
            <Input
              placeholder="Search by name, IP address, user, or department..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              size="sm"
            />
          </Box>
          
          <HStack w="100%" gap={4}>
            <Box flex="1">
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Asset Type
              </Text>
              <HStack wrap="wrap" gap={2}>
                <Button
                  size="sm"
                  variant={selectedType === 'all' ? 'solid' : 'outline'}
                  colorScheme={selectedType === 'all' ? 'blue' : 'gray'}
                  onClick={() => setSelectedType('all')}
                >
                  All Types
                </Button>
                {assetTypes.map(type => (
                  <Button
                    key={type}
                    size="sm"
                    variant={selectedType === type ? 'solid' : 'outline'}
                    colorScheme={selectedType === type ? 'blue' : 'gray'}
                    onClick={() => setSelectedType(type)}
                  >
                    {type}
                  </Button>
                ))}
              </HStack>
            </Box>
            
            <Box flex="1">
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Status
              </Text>
              <HStack wrap="wrap" gap={2}>
                <Button
                  size="sm"
                  variant={selectedStatus === 'all' ? 'solid' : 'outline'}
                  colorScheme={selectedStatus === 'all' ? 'blue' : 'gray'}
                  onClick={() => setSelectedStatus('all')}
                >
                  All Status
                </Button>
                {statuses.map(status => (
                  <Button
                    key={status}
                    size="sm"
                    variant={selectedStatus === status ? 'solid' : 'outline'}
                    colorScheme={selectedStatus === status ? 'blue' : 'gray'}
                    onClick={() => setSelectedStatus(status)}
                  >
                    {status}
                  </Button>
                ))}
              </HStack>
            </Box>
          </HStack>
        </VStack>
      </Box>

      {/* Add Asset Form */}
      {showAddAsset && (
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Add New Asset</Heading>
              <Button variant="outline" onClick={() => setShowAddAsset(false)}>
                Cancel
              </Button>
            </HStack>
            
            <SimpleGrid columns={{ base: 1, md: 2 }} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Asset Name
                </Text>
                <Input
                  placeholder="e.g., WS-001-FINANCE"
                  value={newAsset.name}
                  onChange={(e) => setNewAsset({...newAsset, name: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Asset Type
                </Text>
                <HStack wrap="wrap" gap={2}>
                  {assetTypes.map(type => (
                    <Button
                      key={type}
                      size="sm"
                      variant={newAsset.type === type ? 'solid' : 'outline'}
                      colorScheme={newAsset.type === type ? 'blue' : 'gray'}
                      onClick={() => setNewAsset({...newAsset, type: type as any})}
                    >
                      {type}
                    </Button>
                  ))}
                </HStack>
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  IP Address
                </Text>
                <Input
                  placeholder="e.g., 192.168.1.100"
                  value={newAsset.ipAddress}
                  onChange={(e) => setNewAsset({...newAsset, ipAddress: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Department
                </Text>
                <Input
                  placeholder="e.g., Finance"
                  value={newAsset.department}
                  onChange={(e) => setNewAsset({...newAsset, department: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Assigned To
                </Text>
                <Input
                  placeholder="e.g., John Smith"
                  value={newAsset.assignedTo}
                  onChange={(e) => setNewAsset({...newAsset, assignedTo: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Location
                </Text>
                <Input
                  placeholder="e.g., Building A - Floor 2"
                  value={newAsset.location}
                  onChange={(e) => setNewAsset({...newAsset, location: e.target.value})}
                  size="sm"
                />
              </Box>
            </SimpleGrid>
            
            <HStack justify="end">
              <Button colorScheme="blue" onClick={handleAddAsset}>
                Add Asset
              </Button>
            </HStack>
          </VStack>
        </Box>
      )}

      {/* Assets Table */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">Assets ({filteredAssets.length})</Heading>
            <Text fontSize="sm" color="gray.500">
              Showing {filteredAssets.length} of {assets.length} assets
            </Text>
          </HStack>
          
          {/* Table Header */}
          <Box bg="gray.50" p={3} borderRadius="md">
            <HStack>
              <Text fontWeight="medium" fontSize="sm" flex="2">Asset</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Security</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Compliance</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Agent</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
            </HStack>
          </Box>
          
          {/* Table Body */}
          <VStack gap={0}>
            {filteredAssets.map((asset, index) => (
              <Box key={asset.id} w="100%">
                <HStack p={3} _hover={{ bg: 'gray.50' }}>
                  <Box flex="2">
                    <VStack align="start" gap={1}>
                      <HStack>
                        <Text fontWeight="medium" fontSize="sm">
                          {asset.name}
                        </Text>
                        <Badge
                          colorScheme="blue"
                          variant="subtle"
                          fontSize="xs"
                        >
                          {asset.type}
                        </Badge>
                      </HStack>
                      <Text fontSize="xs" color="gray.500">
                        {asset.assignedTo} • {asset.department}
                      </Text>
                      <Text fontSize="xs" color="gray.500">
                        {asset.ipAddress} • {asset.location}
                      </Text>
                    </VStack>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getStatusColor(asset.status)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {asset.status}
                    </Badge>
                    <Text fontSize="xs" color="gray.500" mt={1}>
                      {asset.vulnerabilities} vulns
                    </Text>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getSecurityLevelColor(asset.securityLevel)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {asset.securityLevel}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getComplianceColor(asset.complianceScore)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {asset.complianceScore}%
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={asset.agentInstalled ? 'green' : 'red'}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {asset.agentInstalled ? 'Installed' : 'Missing'}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <HStack gap={1}>
                      <Button size="xs" variant="outline" colorScheme="blue">
                        View
                      </Button>
                      <Button size="xs" variant="outline" colorScheme="gray">
                        Edit
                      </Button>
                    </HStack>
                  </Box>
                </HStack>
                {index < filteredAssets.length - 1 && <Box h="1px" bg="gray.100" />}
              </Box>
            ))}
          </VStack>
          
          {filteredAssets.length === 0 && (
            <Box textAlign="center" py={8}>
              <Text color="gray.500" fontSize="lg">
                No assets found
              </Text>
              <Text color="gray.400" fontSize="sm">
                Try adjusting your search criteria
              </Text>
            </Box>
          )}
        </VStack>
      </Box>
    </VStack>
  );
};

export default AssetManagement;
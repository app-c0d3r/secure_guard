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

interface Employee {
  id: string;
  employeeId: string;
  firstName: string;
  lastName: string;
  email: string;
  phone: string;
  department: string;
  position: string;
  manager: string;
  location: string;
  startDate: string;
  status: 'Active' | 'Inactive' | 'On Leave' | 'Terminated';
  securityClearance: 'Public' | 'Confidential' | 'Secret' | 'Top Secret';
  lastLogin: string;
  assetsAssigned: number;
  complianceTraining: boolean;
}

const EmployeeManagement: React.FC = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedDepartment, setSelectedDepartment] = useState('all');
  const [selectedStatus, setSelectedStatus] = useState('all');
  const [showAddEmployee, setShowAddEmployee] = useState(false);
  const [newEmployee, setNewEmployee] = useState({
    employeeId: '',
    firstName: '',
    lastName: '',
    email: '',
    phone: '',
    department: '',
    position: '',
    manager: '',
    location: '',
  });

  // Mock data - would come from API
  const [employees] = useState<Employee[]>([
    {
      id: '1',
      employeeId: 'EMP001',
      firstName: 'John',
      lastName: 'Smith',
      email: 'john.smith@company.com',
      phone: '+1 (555) 123-4567',
      department: 'Finance',
      position: 'Financial Analyst',
      manager: 'Sarah Johnson',
      location: 'Building A - Floor 2',
      startDate: '2023-03-15',
      status: 'Active',
      securityClearance: 'Confidential',
      lastLogin: '2024-01-20 14:32',
      assetsAssigned: 2,
      complianceTraining: true,
    },
    {
      id: '2',
      employeeId: 'EMP002',
      firstName: 'Sarah',
      lastName: 'Johnson',
      email: 'sarah.johnson@company.com',
      phone: '+1 (555) 234-5678',
      department: 'Finance',
      position: 'Finance Manager',
      manager: 'David Wilson',
      location: 'Building A - Floor 3',
      startDate: '2022-01-10',
      status: 'Active',
      securityClearance: 'Secret',
      lastLogin: '2024-01-20 09:15',
      assetsAssigned: 3,
      complianceTraining: true,
    },
    {
      id: '3',
      employeeId: 'EMP003',
      firstName: 'Mike',
      lastName: 'Chen',
      email: 'mike.chen@company.com',
      phone: '+1 (555) 345-6789',
      department: 'IT',
      position: 'Security Engineer',
      manager: 'Lisa Anderson',
      location: 'Building B - Floor 1',
      startDate: '2023-08-20',
      status: 'On Leave',
      securityClearance: 'Top Secret',
      lastLogin: '2024-01-15 16:45',
      assetsAssigned: 4,
      complianceTraining: false,
    },
  ]);

  const departments = ['Finance', 'IT', 'Sales', 'Marketing', 'HR', 'Operations', 'Legal'];
  const statuses = ['Active', 'Inactive', 'On Leave', 'Terminated'];

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active': return 'green';
      case 'Inactive': return 'gray';
      case 'On Leave': return 'orange';
      case 'Terminated': return 'red';
      default: return 'gray';
    }
  };

  const getClearanceColor = (clearance: string) => {
    switch (clearance) {
      case 'Public': return 'gray';
      case 'Confidential': return 'blue';
      case 'Secret': return 'orange';
      case 'Top Secret': return 'red';
      default: return 'gray';
    }
  };

  const filteredEmployees = employees.filter(employee => {
    const matchesSearch = employee.firstName.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         employee.lastName.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         employee.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         employee.employeeId.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         employee.position.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesDepartment = selectedDepartment === 'all' || employee.department === selectedDepartment;
    const matchesStatus = selectedStatus === 'all' || employee.status === selectedStatus;
    return matchesSearch && matchesDepartment && matchesStatus;
  });

  const handleAddEmployee = () => {
    console.log('Adding employee:', newEmployee);
    setShowAddEmployee(false);
    setNewEmployee({
      employeeId: '',
      firstName: '',
      lastName: '',
      email: '',
      phone: '',
      department: '',
      position: '',
      manager: '',
      location: '',
    });
  };

  return (
    <VStack gap={6} align="stretch">
      {/* Header */}
      <HStack justify="space-between">
        <Box>
          <Heading size="lg">Employee Management</Heading>
          <Text color="gray.600" mt={1}>
            Manage employee records and organizational structure
          </Text>
        </Box>
        <Button colorScheme="blue" onClick={() => setShowAddEmployee(true)}>
          Add New Employee
        </Button>
      </HStack>

      {/* Statistics */}
      <SimpleGrid columns={{ base: 2, md: 5 }} gap={4}>
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="blue.600">
            {employees.length}
          </Text>
          <Text fontSize="sm" color="gray.600">Total Employees</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="green.600">
            {employees.filter(e => e.status === 'Active').length}
          </Text>
          <Text fontSize="sm" color="gray.600">Active</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="orange.600">
            {employees.filter(e => e.status === 'On Leave').length}
          </Text>
          <Text fontSize="sm" color="gray.600">On Leave</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="red.600">
            {employees.filter(e => !e.complianceTraining).length}
          </Text>
          <Text fontSize="sm" color="gray.600">Training Pending</Text>
        </Box>
        
        <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
          <Text fontSize="2xl" fontWeight="bold" color="purple.600">
            {new Set(employees.map(e => e.department)).size}
          </Text>
          <Text fontSize="sm" color="gray.600">Departments</Text>
        </Box>
      </SimpleGrid>

      {/* Filters */}
      <Box bg="white" p={4} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack gap={4}>
          <Box w="100%">
            <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
              Search Employees
            </Text>
            <Input
              placeholder="Search by name, email, employee ID, or position..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              size="sm"
            />
          </Box>
          
          <HStack w="100%" gap={4}>
            <Box flex="1">
              <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                Department
              </Text>
              <HStack wrap="wrap" gap={2}>
                <Button
                  size="sm"
                  variant={selectedDepartment === 'all' ? 'solid' : 'outline'}
                  colorScheme={selectedDepartment === 'all' ? 'blue' : 'gray'}
                  onClick={() => setSelectedDepartment('all')}
                >
                  All Departments
                </Button>
                {departments.map(dept => (
                  <Button
                    key={dept}
                    size="sm"
                    variant={selectedDepartment === dept ? 'solid' : 'outline'}
                    colorScheme={selectedDepartment === dept ? 'blue' : 'gray'}
                    onClick={() => setSelectedDepartment(dept)}
                  >
                    {dept}
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

      {/* Add Employee Form */}
      {showAddEmployee && (
        <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
          <VStack align="stretch" gap={4}>
            <HStack justify="space-between">
              <Heading size="md">Add New Employee</Heading>
              <Button variant="outline" onClick={() => setShowAddEmployee(false)}>
                Cancel
              </Button>
            </HStack>
            
            <SimpleGrid columns={{ base: 1, md: 2 }} gap={4}>
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Employee ID
                </Text>
                <Input
                  placeholder="e.g., EMP004"
                  value={newEmployee.employeeId}
                  onChange={(e) => setNewEmployee({...newEmployee, employeeId: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Department
                </Text>
                <HStack wrap="wrap" gap={2}>
                  {departments.map(dept => (
                    <Button
                      key={dept}
                      size="sm"
                      variant={newEmployee.department === dept ? 'solid' : 'outline'}
                      colorScheme={newEmployee.department === dept ? 'blue' : 'gray'}
                      onClick={() => setNewEmployee({...newEmployee, department: dept})}
                    >
                      {dept}
                    </Button>
                  ))}
                </HStack>
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  First Name
                </Text>
                <Input
                  value={newEmployee.firstName}
                  onChange={(e) => setNewEmployee({...newEmployee, firstName: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Last Name
                </Text>
                <Input
                  value={newEmployee.lastName}
                  onChange={(e) => setNewEmployee({...newEmployee, lastName: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Email
                </Text>
                <Input
                  type="email"
                  value={newEmployee.email}
                  onChange={(e) => setNewEmployee({...newEmployee, email: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Phone
                </Text>
                <Input
                  value={newEmployee.phone}
                  onChange={(e) => setNewEmployee({...newEmployee, phone: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Position
                </Text>
                <Input
                  placeholder="e.g., Software Engineer"
                  value={newEmployee.position}
                  onChange={(e) => setNewEmployee({...newEmployee, position: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Manager
                </Text>
                <Input
                  placeholder="e.g., Sarah Johnson"
                  value={newEmployee.manager}
                  onChange={(e) => setNewEmployee({...newEmployee, manager: e.target.value})}
                  size="sm"
                />
              </Box>
              
              <Box gridColumn={{ base: 'auto', md: 'span 2' }}>
                <Text fontSize="sm" fontWeight="medium" color="gray.700" mb={2}>
                  Location
                </Text>
                <Input
                  placeholder="e.g., Building A - Floor 2"
                  value={newEmployee.location}
                  onChange={(e) => setNewEmployee({...newEmployee, location: e.target.value})}
                  size="sm"
                />
              </Box>
            </SimpleGrid>
            
            <HStack justify="end">
              <Button colorScheme="blue" onClick={handleAddEmployee}>
                Add Employee
              </Button>
            </HStack>
          </VStack>
        </Box>
      )}

      {/* Employees Table */}
      <Box bg="white" p={6} borderRadius="md" border="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Heading size="md">Employees ({filteredEmployees.length})</Heading>
            <Text fontSize="sm" color="gray.500">
              Showing {filteredEmployees.length} of {employees.length} employees
            </Text>
          </HStack>
          
          {/* Table Header */}
          <Box bg="gray.50" p={3} borderRadius="md">
            <HStack>
              <Text fontWeight="medium" fontSize="sm" flex="2">Employee</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Department</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Status</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Clearance</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Assets</Text>
              <Text fontWeight="medium" fontSize="sm" flex="1">Actions</Text>
            </HStack>
          </Box>
          
          {/* Table Body */}
          <VStack gap={0}>
            {filteredEmployees.map((employee, index) => (
              <Box key={employee.id} w="100%">
                <HStack p={3} _hover={{ bg: 'gray.50' }}>
                  <Box flex="2">
                    <VStack align="start" gap={1}>
                      <HStack>
                        <Text fontWeight="medium" fontSize="sm">
                          {employee.firstName} {employee.lastName}
                        </Text>
                        <Badge
                          colorScheme="gray"
                          variant="subtle"
                          fontSize="xs"
                        >
                          {employee.employeeId}
                        </Badge>
                      </HStack>
                      <Text fontSize="xs" color="gray.500">
                        {employee.position}
                      </Text>
                      <Text fontSize="xs" color="gray.500">
                        {employee.email} • Reports to {employee.manager}
                      </Text>
                    </VStack>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme="blue"
                      variant="subtle"
                      fontSize="xs"
                    >
                      {employee.department}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getStatusColor(employee.status)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {employee.status}
                    </Badge>
                    {!employee.complianceTraining && (
                      <Text fontSize="xs" color="red.500" mt={1}>
                        Training Required
                      </Text>
                    )}
                  </Box>
                  
                  <Box flex="1">
                    <Badge
                      colorScheme={getClearanceColor(employee.securityClearance)}
                      variant="subtle"
                      fontSize="xs"
                    >
                      {employee.securityClearance}
                    </Badge>
                  </Box>
                  
                  <Box flex="1">
                    <Text fontSize="sm" color="gray.600">
                      {employee.assetsAssigned} assigned
                    </Text>
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
                {index < filteredEmployees.length - 1 && <Box h="1px" bg="gray.100" />}
              </Box>
            ))}
          </VStack>
          
          {filteredEmployees.length === 0 && (
            <Box textAlign="center" py={8}>
              <Text color="gray.500" fontSize="lg">
                No employees found
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

export default EmployeeManagement;
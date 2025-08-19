import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'

interface ProtectedRouteProps {
  children: React.ReactNode
  requiredPermissions?: string[]
  requiredRoles?: string[]
  requireAnyRole?: boolean // If true, user needs any of the roles, if false, all roles
}

export default function ProtectedRoute({ 
  children, 
  requiredPermissions = [], 
  requiredRoles = [],
  requireAnyRole = true 
}: ProtectedRouteProps) {
  const { isAuthenticated, user, hasPermission, hasAnyRole, hasRole } = useAuthStore()
  const navigate = useNavigate()

  useEffect(() => {
    // Redirect to login if not authenticated
    if (!isAuthenticated) {
      navigate('/login', { replace: true })
      return
    }

    // Check role requirements
    if (requiredRoles.length > 0) {
      const hasRequiredRole = requireAnyRole 
        ? hasAnyRole(requiredRoles)
        : requiredRoles.every(role => hasRole(role))
        
      if (!hasRequiredRole) {
        navigate('/dashboard', { replace: true })
        return
      }
    }

    // Check permission requirements
    if (requiredPermissions.length > 0) {
      const hasRequiredPermissions = requiredPermissions.every(permission => 
        hasPermission(permission)
      )
      
      if (!hasRequiredPermissions) {
        navigate('/dashboard', { replace: true })
        return
      }
    }
  }, [isAuthenticated, user, navigate, requiredPermissions, requiredRoles, requireAnyRole, hasPermission, hasAnyRole, hasRole])

  // Don't render anything while checking authentication
  if (!isAuthenticated) {
    return null
  }

  // Check permissions/roles before rendering
  if (requiredRoles.length > 0) {
    const hasRequiredRole = requireAnyRole 
      ? hasAnyRole(requiredRoles)
      : requiredRoles.every(role => hasRole(role))
      
    if (!hasRequiredRole) {
      return null
    }
  }

  if (requiredPermissions.length > 0) {
    const hasRequiredPermissions = requiredPermissions.every(permission => 
      hasPermission(permission)
    )
    
    if (!hasRequiredPermissions) {
      return null
    }
  }

  return <>{children}</>
}
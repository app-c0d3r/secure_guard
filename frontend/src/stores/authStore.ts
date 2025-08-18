import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import Cookies from 'js-cookie'

export interface User {
  id: string
  username: string
  email: string
  role: 'system_admin' | 'security_analyst' | 'admin' | 'manager' | 'power_user' | 'user' | 'read_only' | 'guest'
  permissions: string[]
  canAccessSecrets: boolean
  canManageUsers: boolean
  canAdminSystem: boolean
  canControlAgents: boolean
  canViewAssets: boolean
}

interface AuthState {
  user: User | null
  isAuthenticated: boolean
  token: string | null
  
  // Actions
  login: (token: string, user: User) => void
  logout: () => void
  updateUser: (user: Partial<User>) => void
  hasPermission: (permission: string) => boolean
  hasRole: (role: string) => boolean
  hasAnyRole: (roles: string[]) => boolean
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      token: null,

      login: (token: string, user: User) => {
        Cookies.set('auth_token', token, { expires: 7 })
        set({ token, user, isAuthenticated: true })
      },

      logout: () => {
        Cookies.remove('auth_token')
        set({ token: null, user: null, isAuthenticated: false })
      },

      updateUser: (userData: Partial<User>) => {
        const { user } = get()
        if (user) {
          set({ user: { ...user, ...userData } })
        }
      },

      hasPermission: (permission: string) => {
        const { user } = get()
        return user?.permissions.includes(permission) || false
      },

      hasRole: (role: string) => {
        const { user } = get()
        return user?.role === role
      },

      hasAnyRole: (roles: string[]) => {
        const { user } = get()
        return user ? roles.includes(user.role) : false
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({ 
        user: state.user, 
        isAuthenticated: state.isAuthenticated,
        token: state.token 
      }),
    }
  )
)
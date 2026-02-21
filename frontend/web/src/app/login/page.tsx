import { redirect } from 'next/navigation';
import { resolveAppLoginUrl } from '@/config/urls';

export default function LoginPage() {
  redirect(resolveAppLoginUrl());
}

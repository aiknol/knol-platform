import { redirect } from 'next/navigation';
import { resolveAppSignupUrl } from '@/config/urls';

export default function SignupPage() {
  redirect(resolveAppSignupUrl());
}

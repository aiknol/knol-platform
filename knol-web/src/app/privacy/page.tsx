import { Metadata } from 'next';
import { pageTitle, SITE } from '@/config/site';

export const metadata: Metadata = {
  title: pageTitle('Privacy Policy'),
  description: 'Privacy Policy for Knol — how we collect, use, and protect your data.',
};

export default function PrivacyPage() {
  return (
    <div className="px-4 sm:px-6 lg:px-8 py-16">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-dark-50 mb-2">Privacy Policy</h1>
        <p className="text-dark-300 mb-12">Effective Date: February 15, 2026</p>

        {/* Introduction */}
        <section className="mb-12">
          <p className="text-dark-300 text-lg leading-relaxed">
            Knol ("we", "us", "our", or "Company") operates the Knol website and services (collectively, the "Service").
            This page informs you of our policies regarding the collection, use, and disclosure of personal data when you use
            our Service and the choices you have associated with that data.
          </p>
        </section>

        {/* Data Collection */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">1. Information Collection</h2>
          <div className="space-y-6 text-dark-300">
            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">1.1 Information You Provide</h3>
              <p>
                We collect information you voluntarily provide when you interact with our Service, including:
              </p>
              <ul className="list-disc list-inside mt-2 space-y-1 ml-2">
                <li>Account information (name, email, organization)</li>
                <li>API keys and credentials you generate</li>
                <li>Memory and knowledge graph data you store</li>
                <li>Messages, feedback, and support inquiries</li>
                <li>Payment information (processed securely via third-party providers)</li>
              </ul>
            </div>

            <div>
              <h3 className="text-lg font-semibold text-dark-100 mb-2">1.2 Automatically Collected Information</h3>
              <p>
                When you access our Service, we automatically collect certain information about your device and usage:
              </p>
              <ul className="list-disc list-inside mt-2 space-y-1 ml-2">
                <li>Log data (IP address, browser type, pages visited, time and date)</li>
                <li>Device information (operating system, device type, unique identifiers)</li>
                <li>Usage analytics (features used, API call patterns, performance metrics)</li>
                <li>Cookies and similar tracking technologies</li>
              </ul>
            </div>
          </div>
        </section>

        {/* Data Storage and Security */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">2. Data Storage and Security</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              Your data is stored securely in encrypted PostgreSQL databases hosted on reliable infrastructure. We implement:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>End-to-end encryption for sensitive data</li>
              <li>TLS/SSL encryption for data in transit</li>
              <li>Regular security audits and penetration testing</li>
              <li>Access controls and role-based permissions</li>
              <li>Automated backups and disaster recovery procedures</li>
            </ul>
            <p className="mt-4">
              However, no method of transmission over the Internet is 100% secure. While we strive to protect your data,
              we cannot guarantee absolute security.
            </p>
          </div>
        </section>

        {/* Data Usage */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">3. Use of Data</h2>
          <p className="text-dark-300 mb-4">We use the collected data for purposes including:</p>
          <ul className="list-disc list-inside space-y-2 text-dark-300 ml-2">
            <li>Providing, maintaining, and improving the Service</li>
            <li>Processing transactions and sending related information</li>
            <li>Sending technical notices, support messages, and administrative updates</li>
            <li>Responding to your comments, questions, and requests</li>
            <li>Monitoring usage patterns to detect and prevent fraud or abuse</li>
            <li>Analyzing aggregated, non-personally identifiable data to improve our Service</li>
            <li>Complying with legal obligations and enforcing our Terms of Service</li>
          </ul>
        </section>

        {/* Third Parties */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">4. Third-Party Services</h2>
          <p className="text-dark-300 mb-4">
            We may share your data with trusted third-party service providers who assist us in operating our website and Service:
          </p>
          <ul className="list-disc list-inside space-y-2 text-dark-300 ml-2">
            <li>Cloud infrastructure providers (hosting, storage, databases)</li>
            <li>Payment processors (for billing and transactions)</li>
            <li>Analytics providers (to understand usage patterns)</li>
            <li>Customer support platforms (to assist with your inquiries)</li>
            <li>Security and monitoring services</li>
          </ul>
          <p className="text-dark-300 mt-4">
            These providers are contractually obligated to use your data only as necessary to provide services to us
            and maintain strict confidentiality.
          </p>
        </section>

        {/* Cookies */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">5. Cookies and Tracking</h2>
          <div className="space-y-4 text-dark-300">
            <p>
              We use cookies and similar technologies to enhance your experience and analyze Service usage. Cookies are
              small data files stored on your device that help us:
            </p>
            <ul className="list-disc list-inside space-y-1 ml-2">
              <li>Remember your preferences and login information</li>
              <li>Understand how you use the Service</li>
              <li>Deliver personalized content and advertising</li>
              <li>Detect and prevent security threats</li>
            </ul>
            <p className="mt-4">
              You can control cookies through your browser settings. However, disabling cookies may limit functionality.
              We do not respond to "Do Not Track" signals at this time.
            </p>
          </div>
        </section>

        {/* User Rights */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">6. Your Rights and Choices</h2>
          <p className="text-dark-300 mb-4">
            Depending on your location (particularly if you are in the EU, California, or other regions with privacy laws),
            you may have the following rights:
          </p>
          <ul className="list-disc list-inside space-y-2 text-dark-300 ml-2">
            <li>Right to access: Request a copy of your personal data</li>
            <li>Right to correction: Request correction of inaccurate data</li>
            <li>Right to deletion: Request deletion of your data (subject to legal holds)</li>
            <li>Right to restrict processing: Limit how we use your data</li>
            <li>Right to data portability: Receive your data in a structured, machine-readable format</li>
            <li>Right to object: Opt-out of specific data processing activities</li>
            <li>Right to withdraw consent: Withdraw consent for processing at any time</li>
          </ul>
          <p className="text-dark-300 mt-4">
            To exercise these rights, contact us at <a href={`mailto:${SITE.contactEmail}`} className="text-brand-400 hover:text-brand-300">{SITE.contactEmail}</a> or call <a href={`tel:${SITE.contactPhone}`} className="text-brand-400 hover:text-brand-300">{SITE.contactPhoneDisplay}</a> with details of your request.
          </p>
        </section>

        {/* Data Retention */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">7. Data Retention</h2>
          <p className="text-dark-300">
            We retain your personal data for as long as your account is active or as needed to provide services.
            When you delete your account, we securely delete your data within 30 days, except where required to retain
            data for legal, accounting, or security purposes. Aggregated, anonymized data may be retained indefinitely.
          </p>
        </section>

        {/* GDPR / CCPA */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">8. International Data Transfers</h2>
          <p className="text-dark-300">
            Your information may be transferred to and maintained on computers and servers outside of your state, province,
            country or other governmental jurisdiction where data protection laws may differ. If you are located outside
            the United States and choose to provide information to us, we transfer the data to the United States and process
            it there. By providing such information, you consent to this transfer.
          </p>
        </section>

        {/* Children */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">9. Children's Privacy</h2>
          <p className="text-dark-300">
            Our Service is not directed to children under the age of 13. We do not knowingly collect personal information
            from children. If we learn that a child under 13 has provided us with personal information, we will delete such
            information promptly. If you believe we have collected information from a child, please contact us immediately.
          </p>
        </section>

        {/* Changes */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">10. Changes to This Privacy Policy</h2>
          <p className="text-dark-300">
            We may update this Privacy Policy from time to time. Changes will be posted on this page with an updated
            "Effective Date" at the top. Your continued use of the Service after such modifications constitutes acceptance
            of the updated Privacy Policy. We encourage you to review this policy periodically.
          </p>
        </section>

        {/* Contact */}
        <section className="mb-12">
          <h2 className="text-2xl font-bold text-dark-50 mb-4">11. Contact Us</h2>
          <p className="text-dark-300 mb-4">
            If you have questions about this Privacy Policy or our privacy practices, please contact us at:
          </p>
          <div className="bg-dark-700/30 border border-dark-600 rounded-lg p-6 text-dark-300">
            <p className="font-semibold text-dark-100 mb-2">Knol</p>
            <p className="mb-1">Email: <a href={`mailto:${SITE.contactEmail}`} className="text-brand-400 hover:text-brand-300">{SITE.contactEmail}</a></p>
            <p className="mb-1">Phone: <a href={`tel:${SITE.contactPhone}`} className="text-brand-400 hover:text-brand-300">{SITE.contactPhoneDisplay}</a></p>
            <p>GitHub: <a href={SITE.github} className="text-brand-400 hover:text-brand-300" target="_blank" rel="noopener noreferrer">github.com/aiknol/knol</a></p>
          </div>
        </section>
      </div>
    </div>
  );
}
